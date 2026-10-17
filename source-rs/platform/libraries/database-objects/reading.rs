//! Authenticate the head's event and manifest, then only the chunks intersecting
//! a requested range. Read transactions pin one revision across the entire read.
use super::*;
use cathedral_content_layout::{self as layout, digest};
use objects::head_of;
use records::{decode_pin, event_key, key};
use redb::{ReadableDatabase, ReadableTable};
impl Store<'_> {
    /// `expected` zero accepts the current revision; otherwise it must match.
    pub fn read(
        &self,
        root: u8,
        id: u64,
        expected: u64,
        offset: usize,
        out: &mut [u8],
    ) -> Result<Head> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        let head = head_of(&table, root, id)?;
        if expected != 0 && expected != head.revision {
            return Err(Error::Conflict);
        }
        contents(&table, root, id, head, offset, out)?;
        Ok(head)
    }
    pub fn read_pinned(
        &self,
        root: u8,
        slot: u8,
        id: u64,
        revision: u64,
        offset: usize,
        out: &mut [u8],
    ) -> Result<Head> {
        if slot >= PIN_SLOTS || revision == 0 || id == 0 {
            return Err(Error::Invalid);
        }
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        let pin = engine(table.get(key(root, b'p', &[slot]).as_slice()))?.ok_or(Error::Conflict)?;
        let pinned = decode_pin(pin.value())?;
        if pinned.id != id || pinned.head.revision != revision {
            return Err(Error::Conflict);
        }
        contents(&table, root, id, pinned.head, offset, out)?;
        Ok(pinned.head)
    }
}
fn contents(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
    id: u64,
    head: Head,
    offset: usize,
    out: &mut [u8],
) -> Result<()> {
    let event =
        engine(table.get(event_key(root, id, head.revision).as_slice()))?.ok_or(Error::Corrupt)?;
    if event.value() != head.encode() {
        return Err(Error::Corrupt);
    }
    let manifest =
        engine(table.get(key(root, b'm', &head.manifest).as_slice()))?.ok_or(Error::Corrupt)?;
    let body = records::manifest(head, manifest.value())?;
    let end = offset.checked_add(out.len()).ok_or(Error::Invalid)?;
    if end > head.len {
        return Err(Error::Invalid);
    }
    match body {
        records::Contents::Inline(bytes) => out.copy_from_slice(&bytes[offset..end]),
        records::Contents::Chunks(parts) => {
            if out.is_empty() {
                return Ok(());
            }
            for index in offset / CHUNK_BYTES..end.div_ceil(CHUNK_BYTES) {
                let id = &parts[index * 32..(index + 1) * 32];
                let part =
                    engine(table.get(key(root, b'c', id).as_slice()))?.ok_or(Error::Corrupt)?;
                let part = part.value();
                let base = index * CHUNK_BYTES;
                if part.len() != (head.len - base).min(CHUNK_BYTES)
                    || digest(layout::CHUNK_DOMAIN, part) != id
                {
                    return Err(Error::Corrupt);
                }
                let from = offset.max(base);
                let to = end.min(base + part.len());
                out[from - offset..to - offset].copy_from_slice(&part[from - base..to - base]);
            }
        }
    }
    Ok(())
}
