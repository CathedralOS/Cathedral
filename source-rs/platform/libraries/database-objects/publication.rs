//! Admit before hashing; stage immutable records; publish head, event and quota
//! in the same immediate transaction. Any refusal drops the uncommitted writer.
use super::*;
use cathedral_content_layout::{self as layout, Contents, digest};
use objects::{allocate, check_parent, ensure, head_of};
use records::{edge_key, event_key, head_key, key, quota, retain, set_quota};
use redb::ReadableTable;

impl Store<'_> {
    /// Object number zero creates a new object under `parent`; otherwise
    /// `expected` must equal the object's current revision and `parent` is
    /// zero, because publication never moves an object.
    pub fn publish(
        &self,
        root: u8,
        id: u64,
        expected: u64,
        bytes: &[u8],
        parent: u64,
    ) -> Result<Object> {
        self.publish_observed(root, id, expected, bytes, parent, |_| {})
    }
    pub fn publish_observed(
        &self,
        root: u8,
        id: u64,
        expected: u64,
        bytes: &[u8],
        parent: u64,
        observe: impl FnMut(Stage),
    ) -> Result<Object> {
        self.publication(root, id, expected, bytes, parent, None, observe)
    }
    /// Create a named entry and its initial content in one durable transaction.
    /// A collision refuses the entire creation, including allocation and quota.
    pub fn create_named(
        &self,
        root: u8,
        parent: u64,
        metadata: Metadata,
        bytes: &[u8],
    ) -> Result<Object> {
        self.create_named_observed(root, parent, metadata, bytes, |_| {})
    }
    pub fn create_named_observed(
        &self,
        root: u8,
        parent: u64,
        metadata: Metadata,
        bytes: &[u8],
        observe: impl FnMut(Stage),
    ) -> Result<Object> {
        if metadata.name == [0; 32] || Metadata::decode(&metadata.encode()) != Some(metadata) {
            return Err(Error::Invalid);
        }
        self.publication(root, 0, 0, bytes, parent, Some(metadata), observe)
    }
    #[allow(clippy::too_many_arguments)]
    fn publication(
        &self,
        root: u8,
        id: u64,
        expected: u64,
        bytes: &[u8],
        parent: u64,
        metadata: Option<Metadata>,
        mut observe: impl FnMut(Stage),
    ) -> Result<Object> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::TooLarge);
        }
        if (id == 0 && expected != 0) || (id != 0 && parent != 0) {
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
                check_parent(&table, root, parent)?;
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
                retain(&mut table, &key(root, b'k', digest), &[], &mut charged)?;
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
                parent: old.map_or(parent, |o| o.parent),
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
            if old.is_none() {
                ensure(
                    &mut table,
                    &edge_key(root, head.parent, id),
                    &[],
                    &mut charged,
                )?;
            }
            if let Some(metadata) = metadata {
                hierarchy::write(&mut table, root, id, head, metadata, &mut charged)?;
            }
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
