//! Authenticate the current manifest/event and only the chunks intersecting a
//! requested range. Read transactions pin one revision across the entire read.
use super::*;
use cathedral_content_layout::{self as layout, digest};
use records::key;
use redb::{ReadableDatabase, ReadableTable};
impl Store<'_> {
    pub fn read(&self, root: u8, expected: u64, offset: usize, out: &mut [u8]) -> Result<Head> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        let Some(record) = engine(table.get(key(root, b'h', &[]).as_slice()))? else {
            return if expected == 0 && offset == 0 && out.is_empty() {
                Ok(Head::default())
            } else {
                Err(Error::Conflict)
            };
        };
        let head = Head::decode(record.value())?;
        if expected != 0 && expected != head.revision {
            return Err(Error::Conflict);
        }
        let event = engine(table.get(key(root, b'e', &head.revision.to_be_bytes()).as_slice()))?
            .ok_or(Error::Corrupt)?;
        if event.value() != record.value() {
            return Err(Error::Corrupt);
        }
        let manifest =
            engine(table.get(key(root, b'm', &head.manifest).as_slice()))?.ok_or(Error::Corrupt)?;
        let bytes = manifest.value();
        if digest(layout::MANIFEST_DOMAIN, bytes) != head.manifest || bytes.len() < 17 {
            return Err(Error::Corrupt);
        }
        let len = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
        let chunk = u32::from_le_bytes(bytes[9..13].try_into().unwrap()) as usize;
        let count = u32::from_le_bytes(bytes[13..17].try_into().unwrap()) as usize;
        if len != head.len as u64 || chunk != CHUNK_BYTES {
            return Err(Error::Corrupt);
        }
        let end = offset.checked_add(out.len()).ok_or(Error::Invalid)?;
        if end > head.len {
            return Err(Error::Invalid);
        }
        match bytes[0] {
            0 if head.len <= INLINE_BYTES && count == 0 && bytes.len() == 17 + head.len => {
                out.copy_from_slice(&bytes[17 + offset..17 + end])
            }
            1 if head.len > INLINE_BYTES
                && count == head.len.div_ceil(CHUNK_BYTES)
                && bytes.len() == 17 + count * 32 =>
            {
                if out.is_empty() {
                    return Ok(head);
                }
                for index in offset / CHUNK_BYTES..end.div_ceil(CHUNK_BYTES) {
                    let id = &bytes[17 + index * 32..17 + (index + 1) * 32];
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
            _ => return Err(Error::Corrupt),
        }
        Ok(head)
    }
}
