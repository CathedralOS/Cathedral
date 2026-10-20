//! Root-owned revision pins and bounded reachability collection. Every pass
//! derives liveness from all current heads and durable pins under one writer.
//! No persisted cursor can skip newly live records between passes. A pass
//! reads only metadata: heads, events, manifests and the chunk index. Chunk
//! bytes are never read or re-hashed here, so a root full of programs costs a
//! write no more than a root full of notes; reads verify chunks when they
//! are used.
use super::*;
use alloc::{collections::BTreeSet, vec::Vec};
use records::{charge, decode_pin, encode_pin, event_key, key, quota, set_quota};
use redb::{ReadableTable, Table};
pub const PIN_SLOTS: u8 = 2;
const PASS_RECORDS: usize = 8;
#[derive(Debug, Default)]
pub struct Collection {
    pub removed: usize,
    pub charged: u64,
    pub more: bool,
}

impl Store<'_> {
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
            let mut live = BTreeSet::new();
            closure(&table, root, id, head, &mut live)?;
            validate_chunks(&table, root, &live)?;
            let value = encode_pin(Object { id, head });
            let mut charged = quota(&table, root)?;
            charge(&mut charged, pin.len() + value.len())?;
            engine(table.insert(pin.as_slice(), value.as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        engine(tx.commit())
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
        engine(tx.commit())
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
            engine(tx.commit())?;
        }
        Ok(result)
    }
}
/// Run the same bounded retention policy inside a caller-owned transaction.
/// Offline program upgrades use this after staging every replacement, so quota
/// reclamation and the new heads either commit together or roll back together.
/// The caller must preserve the quota ledger and commit with durable semantics.
pub fn collect_records(table: &mut Table<&[u8], &[u8]>, root: u8) -> Result<Collection> {
    let mut live = BTreeSet::new();
    let mut heads = Vec::new();
    for row in engine(table.range(key(root, b'h', &[]).as_slice()..))? {
        let (k, v) = engine(row)?;
        let k = k.value();
        if k.len() != 10 || k[0] != root || k[1] != b'h' {
            break;
        }
        heads.push((
            u64::from_be_bytes(k[2..].try_into().unwrap()),
            Head::decode(v.value())?,
        ));
    }
    for (id, head) in heads {
        closure(table, root, id, head, &mut live)?;
    }
    for slot in 0..PIN_SLOTS {
        if let Some(pin) = engine(table.get(key(root, b'p', &[slot]).as_slice()))? {
            let pinned = decode_pin(pin.value())?;
            closure(table, root, pinned.id, pinned.head, &mut live)?;
        }
    }
    // Validate the distinct live chunk names in index order. Random lookups for
    // every occurrence in every manifest thrash a small engine cache, especially
    // when installed program variants share most of their chunks.
    validate_chunks(table, root, &live)?;
    // Retire event lookup authority before manifests, and manifests
    // before chunks. A pin between passes must never resurrect a
    // partially collected revision. The order of kinds scanned is the
    // retirement order; the chunk index stands in for the chunks.
    let mut retired = Vec::new();
    let mut inspected = 0usize;
    let mut more = false;
    'kinds: for kind in [b'e', b'm', b'k'] {
        for row in engine(table.range(key(root, kind, &[]).as_slice()..))? {
            let (k, v) = engine(row)?;
            let k = k.value();
            if k.len() < 2 || k[0] != root || k[1] != kind {
                break;
            }
            inspected = inspected
                .checked_add(k.len() + v.value().len())
                .ok_or(Error::Corrupt)?;
            if inspected > ROOT_QUOTA as usize {
                return Err(Error::Corrupt);
            }
            if live.contains(k) {
                continue;
            }
            if retired.len() == PASS_RECORDS {
                more = true;
                break 'kinds;
            }
            retired.push(k.to_vec());
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
    Ok(Collection {
        removed: retired.len(),
        charged,
        more,
    })
}
/// Mark one revision's event, manifest and chunk index entries live. The
/// manifest is read and authenticated; chunks are named by it, not read.
fn closure(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    id: u64,
    head: Head,
    live: &mut BTreeSet<Vec<u8>>,
) -> Result<()> {
    let event = event_key(root, id, head.revision);
    if engine(table.get(event.as_slice()))?
        .ok_or(Error::Corrupt)?
        .value()
        != head.encode()
    {
        return Err(Error::Corrupt);
    }
    live.insert(event);
    let manifest = key(root, b'm', &head.manifest);
    if live.contains(&manifest) {
        return Ok(()); // Shared content was already validated through another head.
    }
    let value = engine(table.get(manifest.as_slice()))?.ok_or(Error::Corrupt)?;
    let data = value.value();
    let body = records::manifest(head, data)?;
    if let records::Contents::Chunks(parts) = body {
        for id in parts.chunks_exact(32) {
            let index = key(root, b'k', id);
            live.insert(index);
        }
    }
    live.insert(manifest);
    Ok(())
}

/// Prove every referenced chunk has its index entry before pinning or deleting
/// any records. Only index metadata is scanned; payload bytes remain untouched.
fn validate_chunks(table: &Table<&[u8], &[u8]>, root: u8, live: &BTreeSet<Vec<u8>>) -> Result<()> {
    let expected = live.iter().filter(|key| key.get(1) == Some(&b'k')).count();
    if expected == 0 {
        return Ok(());
    }
    let mut found = 0;
    let mut inspected = 0;
    for row in engine(table.range(key(root, b'k', &[]).as_slice()..))? {
        let (k, v) = engine(row)?;
        let k = k.value();
        if k.len() < 2 || k[..2] != [root, b'k'] {
            break;
        }
        inspected += k.len() + v.value().len();
        if inspected > ROOT_QUOTA as usize || k.len() != 34 || !v.value().is_empty() {
            return Err(Error::Corrupt);
        }
        found += usize::from(live.contains(k));
    }
    if found != expected {
        return Err(Error::Corrupt);
    }
    Ok(())
}
