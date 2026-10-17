//! Root-owned experimental revision pins and bounded reachability collection.
//! Every pass derives liveness from the current head and durable pins under one
//! writer. No persisted cursor can skip newly live records between passes.
use super::*;
use alloc::vec::Vec;
use cathedral_content_layout::{self as layout, digest};
use records::key;
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
    /// Pins are persistent root-owned lab snapshots, not process-local handles.
    /// Only a retained revision can be pinned; occupied slots are never replaced.
    pub fn pin(&self, root: u8, revision: u64, slot: u8) -> Result<()> {
        if slot >= PIN_SLOTS || revision == 0 {
            return Err(Error::Invalid);
        }
        let mut tx = engine(self.db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let pin = key(root, b'p', &[slot]);
            if engine(table.get(pin.as_slice()))?.is_some() {
                return Err(Error::Conflict);
            }
            let head = {
                let value = engine(table.get(key(root, b'e', &revision.to_be_bytes()).as_slice()))?
                    .ok_or(Error::Conflict)?;
                Head::decode(value.value())?
            };
            if head.revision != revision {
                return Err(Error::Corrupt);
            }
            closure(&table, root, head, &mut Vec::new())?;
            let charged = quota(&table, root)?
                .checked_add(51)
                .filter(|n| *n <= ROOT_QUOTA)
                .ok_or(Error::Capacity)?;
            engine(table.insert(pin.as_slice(), head.encode().as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        engine(tx.commit())
    }
    pub fn release(&self, root: u8, revision: u64, slot: u8) -> Result<()> {
        if slot >= PIN_SLOTS || revision == 0 {
            return Err(Error::Invalid);
        }
        let mut tx = engine(self.db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let pin = key(root, b'p', &[slot]);
            let head = engine(table.get(pin.as_slice()))?.ok_or(Error::Conflict)?;
            if Head::decode(head.value())?.revision != revision {
                return Err(Error::Conflict);
            }
            drop(head);
            let charged = quota(&table, root)?.checked_sub(51).ok_or(Error::Corrupt)?;
            engine(table.remove(pin.as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        engine(tx.commit())
    }
    /// At most eight removals and ROOT_QUOTA bytes in the root record scan per
    /// pass. A crash leaves a valid head/pin closure and an exact quota ledger.
    /// Before that scan, validate at most three live object closures (head/pins).
    /// This frees engine pages for reuse; file shrinking is a separate operation.
    pub fn collect(&self, root: u8) -> Result<Collection> {
        let mut tx = engine(self.db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        let result;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let mut live = Vec::new();
            if let Some(head) = engine(table.get(key(root, b'h', &[]).as_slice()))? {
                closure(&table, root, Head::decode(head.value())?, &mut live)?;
            }
            for slot in 0..PIN_SLOTS {
                if let Some(head) = engine(table.get(key(root, b'p', &[slot]).as_slice()))? {
                    closure(&table, root, Head::decode(head.value())?, &mut live)?;
                }
            }
            let mut retired = Vec::new();
            let mut inspected = 0usize;

            let mut more = false;
            for row in engine(table.range([root].as_slice()..))? {
                let (k, v) = engine(row)?;
                let k = k.value();
                if k.first() != Some(&root) {
                    break;
                }
                inspected = inspected
                    .checked_add(k.len() + v.value().len())
                    .ok_or(Error::Corrupt)?;
                if inspected > ROOT_QUOTA as usize || k.len() < 2 {
                    return Err(Error::Corrupt);
                }
                match k[1] {
                    b'h' | b'q' | b'p' => continue,
                    b'c' | b'm' | b'e' => {}
                    _ => return Err(Error::Corrupt),
                }
                if live.iter().any(|l: &Vec<u8>| l.as_slice() == k) {
                    continue;
                }
                // Retire event lookup authority before its manifest/chunks. A
                // pin between passes must never resurrect a partially collected revision.
                retired.push(k.to_vec());
                retired.sort_unstable_by_key(|k| match k[1] {
                    b'e' => 0,
                    b'm' => 1,
                    _ => 2,
                });
                if retired.len() > PASS_RECORDS {
                    retired.pop();
                    more = true;
                }
            }
            let mut reclaimed = 0u64;
            for key in &retired {
                let old = engine(table.remove(key.as_slice()))?.ok_or(Error::Corrupt)?;
                reclaimed += (key.len() + old.value().len()) as u64;
            }
            let charged = quota(&table, root)?
                .checked_sub(reclaimed)
                .ok_or(Error::Corrupt)?;
            if !retired.is_empty() {
                set_quota(&mut table, root, charged)?;
            }
            result = Collection {
                removed: retired.len(),
                charged,
                more,
            };
        }
        if result.removed != 0 {
            engine(tx.commit())?;
        }
        Ok(result)
    }
}
fn quota(table: &Table<&[u8], &[u8]>, root: u8) -> Result<u64> {
    match engine(table.get(key(root, b'q', &[]).as_slice()))? {
        None => Ok(0),
        Some(value) if value.value().len() == 8 => {
            Ok(u64::from_le_bytes(value.value().try_into().unwrap()))
        }
        _ => Err(Error::Corrupt),
    }
}
fn set_quota(table: &mut Table<&[u8], &[u8]>, root: u8, value: u64) -> Result<()> {
    engine(table.insert(
        key(root, b'q', &[]).as_slice(),
        value.to_le_bytes().as_slice(),
    ))?;
    Ok(())
}
fn closure(
    table: &Table<&[u8], &[u8]>,
    root: u8,
    head: Head,
    live: &mut Vec<Vec<u8>>,
) -> Result<()> {
    let event = key(root, b'e', &head.revision.to_be_bytes());
    if engine(table.get(event.as_slice()))?
        .ok_or(Error::Corrupt)?
        .value()
        != head.encode()
    {
        return Err(Error::Corrupt);
    }
    live.push(event);
    let manifest = key(root, b'm', &head.manifest);
    let value = engine(table.get(manifest.as_slice()))?.ok_or(Error::Corrupt)?;
    let data = value.value();
    let body = records::manifest(head, data)?;
    if let records::Contents::Chunks(parts) = body {
        for (index, id) in parts.chunks_exact(32).enumerate() {
            let k = key(root, b'c', id);
            let v = engine(table.get(k.as_slice()))?.ok_or(Error::Corrupt)?;
            if v.value().len() != (head.len - index * CHUNK_BYTES).min(CHUNK_BYTES)
                || digest(layout::CHUNK_DOMAIN, v.value()) != id
            {
                return Err(Error::Corrupt);
            }
            live.push(k);
        }
    }
    live.push(manifest);
    Ok(())
}
