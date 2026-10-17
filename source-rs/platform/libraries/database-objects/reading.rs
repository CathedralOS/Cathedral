//! Authenticate the current manifest/event and only the chunks intersecting a
//! requested range. Read transactions pin one revision across the entire read.
use super::*;
use cathedral_content_layout::{self as layout, digest};
use records::key;
use redb::{ReadableDatabase, ReadableTable};
impl Store<'_> {
    pub fn read(&self, root: u8, expected: u64, offset: usize, out: &mut [u8]) -> Result<Head> {
        self.read_record(root, None, expected, offset, out)
    }
    pub fn read_pinned(
        &self,
        root: u8,
        slot: u8,
        revision: u64,
        offset: usize,
        out: &mut [u8],
    ) -> Result<Head> {
        if slot >= PIN_SLOTS || revision == 0 {
            return Err(Error::Invalid);
        }
        self.read_record(root, Some(slot), revision, offset, out)
    }
    fn read_record(
        &self,
        root: u8,
        pin: Option<u8>,
        expected: u64,
        offset: usize,
        out: &mut [u8],
    ) -> Result<Head> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        let record_key = match pin {
            Some(slot) => key(root, b'p', &[slot]),
            None => key(root, b'h', &[]),
        };
        let Some(record) = engine(table.get(record_key.as_slice()))? else {
            return if pin.is_none() && expected == 0 && offset == 0 && out.is_empty() {
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
        let body = records::manifest(head, manifest.value())?;
        let end = offset.checked_add(out.len()).ok_or(Error::Invalid)?;
        if end > head.len {
            return Err(Error::Invalid);
        }
        match body {
            records::Contents::Inline(bytes) => out.copy_from_slice(&bytes[offset..end]),
            records::Contents::Chunks(parts) => {
                if out.is_empty() {
                    return Ok(head);
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
        Ok(head)
    }
}
