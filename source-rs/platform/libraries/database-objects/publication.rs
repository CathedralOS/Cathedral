//! Admit before hashing; stage immutable records; publish head, event and quota
//! in the same immediate transaction. Any refusal drops the uncommitted writer.
use super::*;
use cathedral_content_layout::{self as layout, Contents, digest};
use records::key;
use redb::{ReadableTable, Table};

impl Store<'_> {
    pub fn publish(&self, root: u8, expected: u64, bytes: &[u8]) -> Result<Head> {
        self.publish_observed(root, expected, bytes, |_| {})
    }
    pub fn publish_observed(
        &self,
        root: u8,
        expected: u64,
        bytes: &[u8],
        mut observe: impl FnMut(Stage),
    ) -> Result<Head> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::TooLarge);
        }
        // Cheap stale requests stop before preparation. The writer rechecks this
        // snapshot; no correctness depends on its remaining current.
        if self.read(root, 0, 0, &mut [])?.revision != expected {
            return Err(Error::Conflict);
        }
        let parts: alloc::vec::Vec<_> = if bytes.len() > INLINE_BYTES {
            bytes
                .chunks(CHUNK_BYTES)
                .map(|part| digest(layout::CHUNK_DOMAIN, part))
                .collect()
        } else {
            alloc::vec::Vec::new()
        };
        let body = if bytes.len() <= INLINE_BYTES {
            Contents::Inline(bytes)
        } else {
            Contents::Chunks(&parts)
        };
        let encoded = layout::manifest(bytes.len(), CHUNK_BYTES as u32, body);
        let id = digest(layout::MANIFEST_DOMAIN, &encoded);
        observe(Stage::Prepared);
        let mut tx = engine(self.db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        let head;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let old = engine(table.get(key(root, b'h', &[]).as_slice()))?
                .map(|r| Head::decode(r.value()))
                .transpose()?
                .unwrap_or_default();
            if old.revision != expected {
                return Err(Error::Conflict);
            }
            let quota_key = key(root, b'q', &[]);
            let mut charged = match engine(table.get(quota_key.as_slice()))? {
                None if old.revision == 0 => 0,
                Some(value) if value.value().len() == 8 => {
                    u64::from_le_bytes(value.value().try_into().unwrap())
                }
                _ => return Err(Error::Corrupt),
            };
            for (part, id) in bytes.chunks(CHUNK_BYTES).zip(&parts) {
                retain(
                    &mut table,
                    key(root, b'c', id).as_slice(),
                    part,
                    &mut charged,
                )?;
            }
            retain(
                &mut table,
                key(root, b'm', &id).as_slice(),
                &encoded,
                &mut charged,
            )?;
            head = Head {
                revision: old.revision.checked_add(1).ok_or(Error::Capacity)?,
                manifest: id,
                len: bytes.len(),
            };
            let event_key = key(root, b'e', &head.revision.to_be_bytes());
            retain(&mut table, &event_key, &head.encode(), &mut charged)?;
            if old.revision == 0 {
                charge(&mut charged, 2 + 48 + 2 + 8)?;
            }
            engine(table.insert(key(root, b'h', &[]).as_slice(), head.encode().as_slice()))?;
            engine(table.insert(quota_key.as_slice(), charged.to_le_bytes().as_slice()))?;
        }
        observe(Stage::Committing);
        engine(tx.commit())?; // No success response before durable publication.
        observe(Stage::Committed);
        Ok(head)
    }
}
fn charge(charged: &mut u64, bytes: usize) -> Result<()> {
    *charged = charged.checked_add(bytes as u64).ok_or(Error::Capacity)?;
    if *charged > ROOT_QUOTA {
        return Err(Error::Capacity);
    }
    Ok(())
}
fn retain(
    table: &mut Table<&[u8], &[u8]>,
    key: &[u8],
    bytes: &[u8],
    charged: &mut u64,
) -> Result<()> {
    if let Some(existing) = engine(table.get(key))? {
        return if existing.value() == bytes {
            Ok(())
        } else {
            Err(Error::Corrupt)
        };
    }
    charge(charged, key.len() + bytes.len())?;
    engine(table.insert(key, bytes))?;
    Ok(())
}
