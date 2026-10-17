//! Admit before hashing; stage immutable records; publish head, event and quota
//! in the same immediate transaction. Any refusal drops the uncommitted writer.
use super::*;
use cathedral_content_layout::{self as layout, Contents, digest};
use objects::{allocate, ensure, head_of};
use records::{event_key, head_key, key, quota, retain, set_quota};
use redb::ReadableTable;

impl Store<'_> {
    /// Object number zero creates a new object; otherwise `expected` must equal
    /// the object's current revision.
    pub fn publish(&self, root: u8, id: u64, expected: u64, bytes: &[u8]) -> Result<Object> {
        self.publish_observed(root, id, expected, bytes, |_| {})
    }
    pub fn publish_observed(
        &self,
        root: u8,
        id: u64,
        expected: u64,
        bytes: &[u8],
        mut observe: impl FnMut(Stage),
    ) -> Result<Object> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::TooLarge);
        }
        if id == 0 && expected != 0 {
            return Err(Error::Invalid);
        }
        // Cheap stale requests stop before preparation. The writer rechecks this
        // snapshot; no correctness depends on its remaining current.
        if id != 0 && self.head(root, id)?.revision != expected {
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
        let manifest = digest(layout::MANIFEST_DOMAIN, &encoded);
        observe(Stage::Prepared);
        let tx = self.writer()?;
        let object;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let old = if id == 0 {
                None
            } else {
                Some(head_of(&table, root, id)?)
            };
            if old.map_or(0, |o| o.revision) != expected {
                return Err(Error::Conflict);
            }
            let mut charged = quota(&table, root)?;
            let id = if id == 0 {
                allocate(&mut table, root, &mut charged)?
            } else {
                id
            };
            for (part, digest) in bytes.chunks(CHUNK_BYTES).zip(&parts) {
                retain(&mut table, &key(root, b'c', digest), part, &mut charged)?;
            }
            retain(
                &mut table,
                &key(root, b'm', &manifest),
                &encoded,
                &mut charged,
            )?;
            let head = Head {
                revision: expected.checked_add(1).ok_or(Error::Capacity)?,
                manifest,
                len: bytes.len(),
            };
            retain(
                &mut table,
                &event_key(root, id, head.revision),
                &head.encode(),
                &mut charged,
            )?;
            ensure(
                &mut table,
                &head_key(root, id),
                &head.encode(),
                &mut charged,
            )?;
            if engine(table.get(key(root, b'q', &[]).as_slice()))?.is_none() {
                records::charge(&mut charged, 2 + 8)?;
            }
            set_quota(&mut table, root, charged)?;
            object = Object { id, head };
        }
        observe(Stage::Committing);
        engine(tx.commit())?; // No success response before durable publication.
        observe(Stage::Committed);
        Ok(object)
    }
}
