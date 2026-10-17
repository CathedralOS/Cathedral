//! Object numbering and head-only operations: look up, list, copy and delete.
//! A copy references the source's manifest; no chunk or manifest is rewritten.
use super::*;
use records::{charge, event_key, head_key, key, quota, retain, set_quota};
use redb::{ReadableDatabase, ReadableTable, Table};

/// Insert a fixed-size structural record, charging it only on first creation.
pub(crate) fn ensure(
    table: &mut Table<&[u8], &[u8]>,
    key: &[u8],
    bytes: &[u8],
    charged: &mut u64,
) -> Result<()> {
    if engine(table.get(key))?.is_none() {
        charge(charged, key.len() + bytes.len())?;
    }
    engine(table.insert(key, bytes))?;
    Ok(())
}
/// Numbers increase monotonically within a root and are never reused, so a
/// stale client holding a deleted number can never reach a newer object.
pub(crate) fn allocate(
    table: &mut Table<&[u8], &[u8]>,
    root: u8,
    charged: &mut u64,
) -> Result<u64> {
    let next_key = key(root, b'n', &[]);
    let id = match engine(table.get(next_key.as_slice()))? {
        None => 1,
        Some(value) if value.value().len() == 8 => {
            u64::from_le_bytes(value.value().try_into().unwrap())
        }
        _ => return Err(Error::Corrupt),
    };
    let next = id.checked_add(1).ok_or(Error::Capacity)?;
    ensure(table, &next_key, &next.to_le_bytes(), charged)?;
    Ok(id)
}
pub(crate) fn head_of(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
    id: u64,
) -> Result<Head> {
    if id == 0 {
        return Err(Error::Invalid);
    }
    match engine(table.get(head_key(root, id).as_slice()))? {
        Some(record) => Head::decode(record.value()),
        None => Err(Error::Missing),
    }
}
impl Store<'_> {
    pub fn head(&self, root: u8, id: u64) -> Result<Head> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        head_of(&table, root, id)
    }
    /// Object numbers greater than `after`, ascending. Returns how many were written.
    pub fn list(&self, root: u8, after: u64, out: &mut [u64]) -> Result<usize> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        let start = head_key(root, after.checked_add(1).ok_or(Error::Invalid)?);
        let mut count = 0;
        for row in engine(table.range(start.as_slice()..))? {
            if count == out.len() {
                break;
            }
            let (k, _) = engine(row)?;
            let k = k.value();
            if k.len() != 10 || k[0] != root || k[1] != b'h' {
                break;
            }
            out[count] = u64::from_be_bytes(k[2..].try_into().unwrap());
            count += 1;
        }
        Ok(count)
    }
    /// A new object whose first revision shares the source's current content.
    /// The write is two small records regardless of the content's size.
    pub fn copy(&self, root: u8, source: u64, expected: u64) -> Result<Object> {
        let tx = self.writer()?;
        let object;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let src = head_of(&table, root, source)?;
            if src.revision != expected {
                return Err(Error::Conflict);
            }
            if engine(table.get(key(root, b'm', &src.manifest).as_slice()))?.is_none() {
                return Err(Error::Corrupt);
            }
            let mut charged = quota(&table, root)?;
            let id = allocate(&mut table, root, &mut charged)?;
            let head = Head {
                revision: 1,
                manifest: src.manifest,
                len: src.len,
            };
            retain(
                &mut table,
                &event_key(root, id, 1),
                &head.encode(),
                &mut charged,
            )?;
            ensure(
                &mut table,
                &head_key(root, id),
                &head.encode(),
                &mut charged,
            )?;
            set_quota(&mut table, root, charged)?;
            object = Object { id, head };
        }
        engine(tx.commit())?;
        Ok(object)
    }
    /// Remove the head. History and content stay until collection finds them
    /// unreachable, so a pinned revision of a deleted object remains readable.
    pub fn delete(&self, root: u8, id: u64, expected: u64) -> Result<()> {
        let tx = self.writer()?;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let head = head_of(&table, root, id)?;
            if head.revision != expected {
                return Err(Error::Conflict);
            }
            let key = head_key(root, id);
            let charged = quota(&table, root)?
                .checked_sub((key.len() + Head::SIZE) as u64)
                .ok_or(Error::Corrupt)?;
            engine(table.remove(key.as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        engine(tx.commit())
    }
}
