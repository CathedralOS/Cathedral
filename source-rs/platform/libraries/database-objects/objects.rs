//! Object numbering, the tree and the head-only operations: look up, list a
//! parent's children, test membership in a subtree, copy one object or a
//! whole subtree, and delete. A copy references the source's manifest; no
//! chunk or manifest is rewritten, so a subtree copy costs three small
//! records per object whatever the content weighs.
use super::*;
use alloc::vec::Vec;
use records::{charge, edge_key, event_key, head_key, key, quota, retain, set_quota};
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
/// A parent must exist before anything is placed under it; zero is the root.
pub(crate) fn check_parent(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
    parent: u64,
) -> Result<()> {
    if parent != 0 {
        head_of(table, root, parent)?;
    }
    Ok(())
}
/// Whether `object` lies at or below `scope`; scope zero is the whole root and
/// object zero is the root itself. Walks parents upward, so an object's own
/// ancestry is the only evidence. A parent always exists before its child and
/// an object never moves, so every parent number is smaller than its child's
/// and the walk terminates; any other shape is corruption.
pub(crate) fn within(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
    mut object: u64,
    scope: u64,
) -> Result<bool> {
    if scope == 0 {
        return Ok(true);
    }
    loop {
        if object == scope {
            return Ok(true);
        }
        if object == 0 {
            return Ok(false);
        }
        let parent = head_of(table, root, object)?.parent;
        if parent >= object {
            return Err(Error::Corrupt);
        }
        object = parent;
    }
}
/// Children of `parent` with numbers above `after`, ascending.
pub(crate) fn children(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
    parent: u64,
    after: u64,
    out: &mut [u64],
) -> Result<usize> {
    let start = edge_key(root, parent, after.checked_add(1).ok_or(Error::Invalid)?);
    let mut count = 0;
    for row in engine(table.range(start.as_slice()..))? {
        if count == out.len() {
            break;
        }
        let (k, _) = engine(row)?;
        let k = k.value();
        if k.len() != 18 || k[0] != root || k[1] != b't' || k[2..10] != parent.to_be_bytes() {
            break;
        }
        out[count] = u64::from_be_bytes(k[10..].try_into().unwrap());
        count += 1;
    }
    Ok(count)
}
/// Write the records that make a new object exist: its head, its first event
/// and the edge from its parent. Content is shared, never written here.
fn place(
    table: &mut Table<&[u8], &[u8]>,
    root: u8,
    head: Head,
    charged: &mut u64,
) -> Result<Object> {
    let id = allocate(table, root, charged)?;
    retain(
        table,
        &event_key(root, id, head.revision),
        &head.encode(),
        charged,
    )?;
    ensure(table, &head_key(root, id), &head.encode(), charged)?;
    ensure(table, &edge_key(root, head.parent, id), &[], charged)?;
    Ok(Object { id, head })
}
impl Store<'_> {
    pub fn head(&self, root: u8, id: u64) -> Result<Head> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        head_of(&table, root, id)
    }
    /// Children of `parent` numbered above `after`, ascending; how many were written.
    pub fn list(&self, root: u8, parent: u64, after: u64, out: &mut [u64]) -> Result<usize> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        check_parent(&table, root, parent)?;
        children(&table, root, parent, after, out)
    }
    /// Direct children after `after` with their current revision and metadata,
    /// in one transaction, so a realm is named once rather than once per child.
    pub fn entries(
        &self,
        root: u8,
        parent: u64,
        after: u64,
        out: &mut [cathedral_platform_contracts::storage::Entry],
    ) -> Result<usize> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        check_parent(&table, root, parent)?;
        let mut ids = [0; cathedral_platform_contracts::storage::ENTRIES_PER_PAGE];
        let limit = out.len().min(ids.len());
        let count = children(&table, root, parent, after, &mut ids[..limit])?;
        for (slot, &id) in out.iter_mut().zip(&ids[..count]) {
            let head = head_of(&table, root, id)?;
            *slot = cathedral_platform_contracts::storage::Entry {
                object: id,
                revision: head.revision,
                metadata: hierarchy::entry(&table, root, id, head)?,
            };
        }
        Ok(count)
    }
    pub fn within(&self, root: u8, object: u64, scope: u64) -> Result<bool> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        within(&table, root, object, scope)
    }
    /// A new object under `parent` whose first revision shares the source's
    /// current content. The write is three small records regardless of size.
    pub fn copy(&self, root: u8, source: u64, expected: u64, parent: u64) -> Result<Object> {
        self.copy_named(root, source, expected, parent, b"")
    }
    /// Copy just the content, optionally naming its new hierarchy entry in the
    /// same transaction. Unlike a subtree copy this may live under its source.
    pub fn copy_named(
        &self,
        root: u8,
        source: u64,
        expected: u64,
        parent: u64,
        name: &[u8],
    ) -> Result<Object> {
        let name = Metadata::new(name, b"").ok_or(Error::Invalid)?.name;
        let tx = self.writer()?;
        let object;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let src = head_of(&table, root, source)?;
            if src.revision != expected {
                return Err(Error::Conflict);
            }
            check_parent(&table, root, parent)?;
            if engine(table.get(key(root, b'm', &src.manifest).as_slice()))?.is_none() {
                return Err(Error::Corrupt);
            }
            let mut charged = quota(&table, root)?;
            let head = Head {
                revision: 1,
                manifest: src.manifest,
                len: src.len,
                parent,
            };
            object = place(&mut table, root, head, &mut charged)?;
            let mut metadata = hierarchy::entry(&table, root, source, src)?;
            metadata.name = name;
            hierarchy::write(&mut table, root, object.id, head, metadata, &mut charged)?;
            set_quota(&mut table, root, charged)?;
        }
        commit(tx)?;
        Ok(object)
    }
    /// Copy `source` and every descendant under `parent`, preserving shape.
    /// Each copy starts at revision one with its original's current content.
    /// Only the source itself is checked against `expected`; descendants are
    /// copied as they are. More than COPY_LIMIT objects is refused whole.
    pub fn copy_tree(&self, root: u8, source: u64, expected: u64, parent: u64) -> Result<Object> {
        self.copy_tree_named(root, source, expected, parent, b"")
    }
    pub fn copy_tree_named(
        &self,
        root: u8,
        source: u64,
        expected: u64,
        parent: u64,
        name: &[u8],
    ) -> Result<Object> {
        let copy_name = Metadata::new(name, b"").ok_or(Error::Invalid)?.name;
        let tx = self.writer()?;
        let copied;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let src = head_of(&table, root, source)?;
            if src.revision != expected {
                return Err(Error::Conflict);
            }
            check_parent(&table, root, parent)?;
            if within(&table, root, parent, source)? {
                return Err(Error::Invalid); // A tree cannot be copied into itself.
            }
            // Enumerate the whole source subtree before writing anything, so
            // the copies never appear in their own enumeration.
            let mut sources: Vec<(u64, Head)> = alloc::vec![(source, src)];
            let mut index = 0;
            while index < sources.len() {
                let (id, _) = sources[index];
                let mut found = [0; COPY_LIMIT];
                let count = children(&table, root, id, 0, &mut found)?;
                for &child in &found[..count] {
                    if sources.len() == COPY_LIMIT {
                        return Err(Error::Capacity);
                    }
                    sources.push((child, head_of(&table, root, child)?));
                }
                index += 1;
            }
            let mut charged = quota(&table, root)?;
            let mut copies: Vec<(u64, u64)> = Vec::new(); // (source number, copy number)
            let mut first = None;
            for (id, head) in sources {
                let new_parent = if id == source {
                    parent
                } else {
                    copies
                        .iter()
                        .find(|(from, _)| *from == head.parent)
                        .map(|(_, to)| *to)
                        .ok_or(Error::Corrupt)?
                };
                if engine(table.get(key(root, b'm', &head.manifest).as_slice()))?.is_none() {
                    return Err(Error::Corrupt);
                }
                let object = place(
                    &mut table,
                    root,
                    Head {
                        revision: 1,
                        manifest: head.manifest,
                        len: head.len,
                        parent: new_parent,
                    },
                    &mut charged,
                )?;
                let mut metadata = hierarchy::entry(&table, root, id, head)?;
                if id == source {
                    metadata.name = copy_name;
                }
                hierarchy::write(
                    &mut table,
                    root,
                    object.id,
                    object.head,
                    metadata,
                    &mut charged,
                )?;
                copies.push((id, object.id));
                first.get_or_insert(object);
            }
            set_quota(&mut table, root, charged)?;
            copied = first.ok_or(Error::Corrupt)?;
        }
        commit(tx)?;
        Ok(copied)
    }
    /// Remove a childless object's head and edge. History and content stay
    /// until collection finds them unreachable, so a pinned revision of a
    /// deleted object remains readable.
    pub fn delete(&self, root: u8, id: u64, expected: u64) -> Result<()> {
        let tx = self.writer()?;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let head = head_of(&table, root, id)?;
            if head.revision != expected {
                return Err(Error::Conflict);
            }
            if children(&table, root, id, 0, &mut [0])? != 0 {
                return Err(Error::NotEmpty);
            }
            let head_key = head_key(root, id);
            let mut charged = quota(&table, root)?;
            hierarchy::clear(&mut table, root, id, head, &mut charged)?;
            let charged = charged
                .checked_sub((head_key.len() + Head::SIZE) as u64)
                .ok_or(Error::Corrupt)?;
            engine(table.remove(head_key.as_slice()))?;
            set_quota(&mut table, root, charged)?;
        }
        commit(tx)
    }
}
