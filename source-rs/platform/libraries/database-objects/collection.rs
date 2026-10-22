//! Root-owned revision pins and bounded reachability collection. Every pass
//! derives liveness from all current heads and durable pins under one writer.
//! No persisted cursor can skip newly live records between passes. A pass
//! reads only metadata: heads, events, manifests and the chunk index. Chunk
//! bytes are never read or re-hashed here: collection work scales with revision
//! and chunk metadata rather than payload bytes. Reads verify chunks when used.
//!
//! Workspace is bounded independently of the root's record count. Liveness is
//! marked one record kind at a time, inside one window of identities. A window
//! whose live marks would exceed the workspace is split in two and marked
//! again, so a larger root costs more metadata passes rather than more heap.
use super::*;
use alloc::{collections::BTreeSet, vec::Vec};
use records::{charge, decode_pin, encode_pin, event_key, head_key, key, quota, set_quota};
use redb::{ReadableTable, Table};
pub const PIN_SLOTS: u8 = 2;
const PASS_RECORDS: usize = 8;
/// Heap bytes one collection pass may hold in live marks. The provider's heap
/// also carries engine state and the writer; this stays well inside the rest.
pub const WORKSPACE_BYTES: usize = 24 * 1024;
/// Conservative heap cost of one marked identity: a 32-byte key inside a
/// B-tree set node, including node headers and partial fill.
const MARK_BYTES: usize = 64;
/// Heads examined per range visit before the range is released and resumed.
const HEAD_BATCH: usize = 32;

/// Identities of one record kind whose bucket byte lies in `first..=last`.
#[derive(Clone, Copy)]
struct Window {
    kind: u8,
    first: u8,
    last: u8,
}
impl Window {
    fn all(kind: u8) -> Self {
        Self {
            kind,
            first: 0,
            last: u8::MAX,
        }
    }
    /// Window `index` of `count` equal windows; `count` is a power of two up to 256.
    fn of(kind: u8, index: u32, count: u32) -> Self {
        let span = 256 / count;
        let first = index * span;
        Self {
            kind,
            first: first as u8,
            last: (first + span - 1) as u8,
        }
    }
    /// Objects are numbered sequentially, so an event's low object byte spreads
    /// evenly; manifests and chunks are digests, so their first byte does.
    fn bucket(kind: u8, suffix: &[u8]) -> Option<u8> {
        match (kind, suffix.len()) {
            (b'e', 16) => Some(suffix[7]),
            (b'm' | b'k', 32) => Some(suffix[0]),
            _ => None,
        }
    }
    fn covers(&self, suffix: &[u8]) -> bool {
        Self::bucket(self.kind, suffix).is_some_and(|b| (self.first..=self.last).contains(&b))
    }
    /// Digest kinds are key-ordered by their bucket; events are not.
    fn ordered(&self) -> bool {
        self.kind != b'e'
    }
}
struct Overflow;
/// Whether a window still has room after an insertion.
type Marked = core::result::Result<(), Overflow>;
/// Live identities of one window, padded to one fixed width. The limit is
/// checked before insertion, so the set never grows past its workspace.
struct Marks {
    ids: BTreeSet<[u8; 32]>,
    limit: usize,
}
impl Marks {
    fn new(workspace: usize) -> Self {
        Self {
            ids: BTreeSet::new(),
            limit: workspace / MARK_BYTES,
        }
    }
    fn pad(suffix: &[u8]) -> [u8; 32] {
        let mut id = [0; 32];
        id[..suffix.len()].copy_from_slice(suffix);
        id
    }
    fn insert(&mut self, suffix: &[u8]) -> Marked {
        let id = Self::pad(suffix);
        if self.ids.contains(&id) {
            return Ok(());
        }
        if self.ids.len() >= self.limit {
            return Err(Overflow);
        }
        self.ids.insert(id);
        Ok(())
    }
    fn contains(&self, suffix: &[u8]) -> bool {
        suffix.len() <= 32 && self.ids.contains(&Self::pad(suffix))
    }
}
#[derive(Debug, Default)]
pub struct Collection {
    pub removed: usize,
    pub charged: u64,
    pub more: bool,
    /// Mark windows this pass computed: four when every kind fits at once.
    pub windows: u32,
    /// Largest number of identities marked in one window.
    pub marked: usize,
}

impl Store<'_> {
    /// Automatic maintenance is pressure-driven. Reserve twice the largest
    /// content publication for its records/indexes and the next bounded copy.
    /// Below this watermark, read only the durable ledger: opening a writer
    /// and walking every live revision on every small save is unnecessary.
    /// This is a trigger, not admission; each mutation still checks exact quota.
    /// Old unpinned revisions are eligible for collection, never guaranteed.
    pub fn collect_under_pressure(&self, root: u8) -> Result<Option<Collection>> {
        let charged = {
            let read = engine(self.db.begin_read())?;
            let table = engine(read.open_table(TABLE))?;
            quota(&table, root)?
        };
        if charged > ROOT_QUOTA {
            return Err(Error::Corrupt);
        }
        if charged < ROOT_QUOTA - 2 * MAX_BYTES as u64 {
            return Ok(None);
        }
        self.collect(root).map(Some)
    }
    /// Pins are persistent root-owned snapshots, not process-local handles.
    /// Only a retained revision can be pinned; occupied slots are never replaced.
    pub fn pin(&self, root: u8, id: u64, revision: u64, slot: u8) -> Result<()> {
        if slot >= PIN_SLOTS || revision == 0 || id == 0 {
            return Err(Error::Invalid);
        }
        let tx = self.writer()?;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let pin = key(root, b'p', &[slot]);
            if engine(table.get(pin.as_slice()))?.is_some() {
                return Err(Error::Conflict);
            }
            let head = {
                let value = engine(table.get(event_key(root, id, revision).as_slice()))?
                    .ok_or(Error::Conflict)?;
                Head::decode(value.value())?
            };
            if head.revision != revision {
                return Err(Error::Corrupt);
            }
            let marks = revision_chunks(&table, root, id, head)?;
            validate_chunks(&table, root, Window::all(b'k'), &marks)?;
            let value = encode_pin(Object { id, head });
            let mut charged = quota(&table, root)?;
            charge(&mut charged, pin.len() + value.len())?;
            engine(table.insert(pin.as_slice(), value.as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        commit(tx)
    }
    pub fn release(&self, root: u8, id: u64, revision: u64, slot: u8) -> Result<()> {
        if slot >= PIN_SLOTS || revision == 0 || id == 0 {
            return Err(Error::Invalid);
        }
        let tx = self.writer()?;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let pin = key(root, b'p', &[slot]);
            let value = engine(table.get(pin.as_slice()))?.ok_or(Error::Conflict)?;
            let pinned = decode_pin(value.value())?;
            if pinned.id != id || pinned.head.revision != revision {
                return Err(Error::Conflict);
            }
            let reclaimed = (pin.len() + value.value().len()) as u64;
            drop(value);
            let charged = quota(&table, root)?
                .checked_sub(reclaimed)
                .ok_or(Error::Corrupt)?;
            engine(table.remove(pin.as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        commit(tx)
    }
    /// At most eight removals; each ordered metadata scan is bounded by ROOT_QUOTA.
    /// A crash leaves every head/pin closure valid and an exact ledger.
    /// This frees engine pages for reuse; file shrinking is a separate operation.
    pub fn collect(&self, root: u8) -> Result<Collection> {
        let tx = self.writer()?;
        let result = {
            let mut table = engine(tx.open_table(TABLE))?;
            collect_records(&mut table, root)?
        };
        if result.removed != 0 {
            commit(tx)?;
        }
        Ok(result)
    }
}
/// Run the same bounded retention policy inside a caller-owned transaction.
/// Offline program upgrades use this after staging every replacement, so quota
/// reclamation and the new heads either commit together or roll back together.
/// The caller must preserve the quota ledger and commit with durable semantics.
pub fn collect_records(table: &mut Table<&[u8], &[u8]>, root: u8) -> Result<Collection> {
    collect_records_within(table, root, WORKSPACE_BYTES)
}
/// The bounded pass with an explicit workspace. Windows are split until each
/// fits; a window that does not fit even at the finest split reports
/// `Error::Workspace` before anything is removed.
pub fn collect_records_within(
    table: &mut Table<&[u8], &[u8]>,
    root: u8,
    workspace: usize,
) -> Result<Collection> {
    let mut result = Collection::default();
    // Prove the chunk index complete before deleting or pinning any record.
    windows(
        table,
        root,
        b'k',
        workspace,
        &mut result,
        |table, window, marks| validate_chunks(table, root, window, marks).map(|()| false),
    )?;
    // Retire event lookup authority before manifests, and manifests before
    // chunks. A pin between passes must never resurrect a partially collected
    // revision. The order of kinds swept is the retirement order; the chunk
    // index stands in for the chunks.
    let mut retired = Vec::new();
    for kind in [b'e', b'm', b'k'] {
        let spent = windows(
            table,
            root,
            kind,
            workspace,
            &mut result,
            |table, window, marks| sweep(table, root, window, marks, &mut retired),
        )?;
        if spent {
            result.more = true;
            break;
        }
    }
    let mut reclaimed = 0u64;
    for key in &retired {
        let old = engine(table.remove(key.as_slice()))?.ok_or(Error::Corrupt)?;
        reclaimed += (key.len() + old.value().len()) as u64;
        drop(old);
        if key[1] == b'k' {
            let chunk = records::key(root, b'c', &key[2..]);
            let old = engine(table.remove(chunk.as_slice()))?.ok_or(Error::Corrupt)?;
            reclaimed += (chunk.len() + old.value().len()) as u64;
        }
    }
    let charged = quota(table, root)?
        .checked_sub(reclaimed)
        .ok_or(Error::Corrupt)?;
    if !retired.is_empty() {
        set_quota(table, root, charged)?;
    }
    result.removed = retired.len();
    result.charged = charged;
    Ok(result)
}
/// Visit one kind through as many windows as the workspace requires. A window
/// that overflows is replaced by its two halves. The visit returns true to end
/// the pass early, once the removal allowance is spent.
fn windows(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    kind: u8,
    workspace: usize,
    result: &mut Collection,
    mut visit: impl FnMut(&Table<&[u8], &[u8]>, Window, &Marks) -> Result<bool>,
) -> Result<bool> {
    let mut count = 1u32;
    let mut index = 0u32;
    while index < count {
        let window = Window::of(kind, index, count);
        let Some(marks) = mark(table, root, window, workspace)? else {
            if count == 256 {
                return Err(Error::Workspace);
            }
            index *= 2;
            count *= 2;
            continue;
        };
        result.windows += 1;
        result.marked = result.marked.max(marks.ids.len());
        if visit(table, window, &marks)? {
            return Ok(true);
        }
        index += 1;
    }
    Ok(false)
}
/// Mark every live identity of the window's kind, or report that the window
/// does not fit the workspace. Heads are visited in bounded batches so no
/// range is held across the lookups each head needs.
fn mark(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    window: Window,
    workspace: usize,
) -> Result<Option<Marks>> {
    let mut marks = Marks::new(workspace - workspace / 4);
    // Manifests already expanded in this chunk window: a dedup aid that stops
    // growing when its share is spent, never a correctness input.
    let mut expanded = Marks::new(workspace / 4);
    let mut next = 0u64;
    loop {
        let mut batch = [(0u64, Head::default()); HEAD_BATCH];
        let mut count = 0;
        for row in engine(table.range(head_key(root, next).as_slice()..))? {
            if count == HEAD_BATCH {
                break;
            }
            let (k, v) = engine(row)?;
            let k = k.value();
            if k.len() != 10 || k[0] != root || k[1] != b'h' {
                break;
            }
            batch[count] = (
                u64::from_be_bytes(k[2..].try_into().unwrap()),
                Head::decode(v.value())?,
            );
            count += 1;
        }
        for &(id, head) in &batch[..count] {
            if mark_revision(table, root, id, head, window, &mut marks, &mut expanded)?.is_err() {
                return Ok(None);
            }
        }
        if count < HEAD_BATCH {
            break;
        }
        next = batch[count - 1].0.checked_add(1).ok_or(Error::Corrupt)?;
    }
    for slot in 0..PIN_SLOTS {
        let Some(pin) = engine(table.get(key(root, b'p', &[slot]).as_slice()))? else {
            continue;
        };
        let pinned = decode_pin(pin.value())?;
        drop(pin);
        if mark_revision(
            table,
            root,
            pinned.id,
            pinned.head,
            window,
            &mut marks,
            &mut expanded,
        )?
        .is_err()
        {
            return Ok(None);
        }
    }
    Ok(Some(marks))
}
/// Mark what one live revision contributes to the window's kind. The event
/// pass authenticates every event against its head before any removal; chunk
/// windows read and authenticate the manifest that names their chunks.
fn mark_revision(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    id: u64,
    head: Head,
    window: Window,
    marks: &mut Marks,
    expanded: &mut Marks,
) -> Result<Marked> {
    match window.kind {
        b'e' => {
            let event = event_key(root, id, head.revision);
            if engine(table.get(event.as_slice()))?
                .ok_or(Error::Corrupt)?
                .value()
                != head.encode()
            {
                return Err(Error::Corrupt);
            }
            Ok(if window.covers(&event[2..]) {
                marks.insert(&event[2..])
            } else {
                Ok(())
            })
        }
        b'm' => Ok(if window.covers(&head.manifest) {
            marks.insert(&head.manifest)
        } else {
            Ok(())
        }),
        _ => {
            if expanded.contains(&head.manifest) {
                return Ok(Ok(())); // Shared content was already expanded through another head.
            }
            let value = engine(table.get(key(root, b'm', &head.manifest).as_slice()))?
                .ok_or(Error::Corrupt)?;
            if let records::Contents::Chunks(parts) = records::manifest(head, value.value())? {
                for chunk in parts.chunks_exact(32) {
                    if window.covers(chunk) && marks.insert(chunk).is_err() {
                        return Ok(Err(Overflow));
                    }
                }
            }
            let _ = expanded.insert(&head.manifest);
            Ok(Ok(()))
        }
    }
}
/// The chunks one revision names, for pinning: at most MAX_BYTES / CHUNK_BYTES.
fn revision_chunks(table: &Table<&[u8], &[u8]>, root: u8, id: u64, head: Head) -> Result<Marks> {
    let mut marks = Marks::new((MAX_BYTES / CHUNK_BYTES + 1) * MARK_BYTES);
    let mut expanded = Marks::new(MARK_BYTES);
    match mark_revision(
        table,
        root,
        id,
        head,
        Window::all(b'k'),
        &mut marks,
        &mut expanded,
    )? {
        Ok(()) => Ok(marks),
        Err(Overflow) => Err(Error::Corrupt),
    }
}
/// Prove every live chunk of the window has its index entry before any record
/// is removed or pinned. Only index metadata is scanned; payload stays untouched.
fn validate_chunks(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    window: Window,
    marks: &Marks,
) -> Result<()> {
    if marks.ids.is_empty() {
        return Ok(());
    }
    let mut found = 0;
    let mut inspected = 0;
    for row in engine(table.range(key(root, b'k', &[window.first]).as_slice()..))? {
        let (k, v) = engine(row)?;
        let k = k.value();
        if k.len() < 3 || k[..2] != [root, b'k'] || k[2] > window.last {
            break;
        }
        inspected += k.len() + v.value().len();
        if inspected > ROOT_QUOTA as usize || k.len() != 34 || !v.value().is_empty() {
            return Err(Error::Corrupt);
        }
        found += usize::from(marks.contains(&k[2..]));
    }
    if found != marks.ids.len() {
        return Err(Error::Corrupt);
    }
    Ok(())
}
/// Queue the window's records that no live revision marks, up to the pass
/// allowance. True means the allowance is spent and more work remains.
fn sweep(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    window: Window,
    marks: &Marks,
    retired: &mut Vec<Vec<u8>>,
) -> Result<bool> {
    let start = if window.ordered() {
        key(root, window.kind, &[window.first])
    } else {
        key(root, window.kind, &[])
    };
    let mut inspected = 0usize;
    for row in engine(table.range(start.as_slice()..))? {
        let (k, v) = engine(row)?;
        let k = k.value();
        if k.len() < 3
            || k[0] != root
            || k[1] != window.kind
            || (window.ordered() && k[2] > window.last)
        {
            break;
        }
        inspected = inspected
            .checked_add(k.len() + v.value().len())
            .ok_or(Error::Corrupt)?;
        if inspected > ROOT_QUOTA as usize {
            return Err(Error::Corrupt);
        }
        if !window.covers(&k[2..]) || marks.contains(&k[2..]) {
            continue;
        }
        if retired.len() == PASS_RECORDS {
            return Ok(true);
        }
        retired.push(k.to_vec());
    }
    Ok(false)
}
