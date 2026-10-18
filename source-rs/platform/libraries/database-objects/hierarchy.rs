//! Entry metadata, exact names and indexed media-type queries. All index edits
//! commit with the edge and revision. Content manifests are never rewritten.
use super::*;
use alloc::vec::Vec;
use cathedral_platform_contracts::storage::metadata::{self, FIELD};
use objects::{check_parent, head_of};
use records::{edge_key, event_key, head_key, key, quota, retain, set_quota};
use redb::{ReadableDatabase, ReadableTable, Table};

pub(crate) fn entry(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
    id: u64,
    head: Head,
) -> Result<Metadata> {
    let value =
        engine(table.get(edge_key(root, head.parent, id).as_slice()))?.ok_or(Error::Corrupt)?;
    Metadata::decode(value.value()).ok_or(Error::Corrupt)
}
fn name_key(root: u8, parent: u64, name: &[u8; FIELD]) -> Vec<u8> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(name);
    key(root, b'd', &suffix)
}
fn type_key(root: u8, parent: u64, media_type: &[u8; FIELD], id: u64) -> Vec<u8> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(media_type);
    suffix.extend_from_slice(&id.to_be_bytes());
    key(root, b'y', &suffix)
}
fn remove(table: &mut Table<&[u8], &[u8]>, key: &[u8], charged: &mut u64) -> Result<()> {
    if let Some(old) = engine(table.remove(key))? {
        *charged = charged
            .checked_sub((key.len() + old.value().len()) as u64)
            .ok_or(Error::Corrupt)?;
    }
    Ok(())
}
pub(crate) fn clear(
    table: &mut Table<&[u8], &[u8]>,
    root: u8,
    id: u64,
    head: Head,
    charged: &mut u64,
) -> Result<()> {
    let old = entry(table, root, id, head)?;
    if old.name != [0; FIELD] {
        remove(table, &name_key(root, head.parent, &old.name), charged)?;
    }
    if old.media_type != [0; FIELD] {
        remove(
            table,
            &type_key(root, head.parent, &old.media_type, id),
            charged,
        )?;
    }
    remove(table, &edge_key(root, head.parent, id), charged)
}
pub(crate) fn write(
    table: &mut Table<&[u8], &[u8]>,
    root: u8,
    id: u64,
    head: Head,
    value: Metadata,
    charged: &mut u64,
) -> Result<()> {
    if Metadata::decode(&value.encode()) != Some(value) {
        return Err(Error::Invalid);
    }
    if value.name != [0; FIELD] {
        let k = name_key(root, head.parent, &value.name);
        if let Some(existing) = engine(table.get(k.as_slice()))?
            && existing.value() != id.to_be_bytes()
        {
            return Err(Error::Conflict);
        }
    }
    clear(table, root, id, head, charged)?;
    let bytes = value.encode();
    // Preserve the legacy empty encoding when there is no metadata.
    retain(
        table,
        &edge_key(root, head.parent, id),
        if value == Metadata::default() {
            &[]
        } else {
            &bytes
        },
        charged,
    )?;
    if value.name != [0; FIELD] {
        retain(
            table,
            &name_key(root, head.parent, &value.name),
            &id.to_be_bytes(),
            charged,
        )?;
    }
    if value.media_type != [0; FIELD] {
        retain(
            table,
            &type_key(root, head.parent, &value.media_type, id),
            &[],
            charged,
        )?;
    }
    Ok(())
}
impl Store<'_> {
    pub fn metadata(&self, root: u8, id: u64) -> Result<(Head, Metadata)> {
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        let head = head_of(&table, root, id)?;
        Ok((head, entry(&table, root, id, head)?))
    }
    pub fn set_metadata(
        &self,
        root: u8,
        id: u64,
        expected: u64,
        value: Metadata,
    ) -> Result<Object> {
        let tx = self.writer()?;
        let object;
        {
            let mut table = engine(tx.open_table(TABLE))?;
            let mut head = head_of(&table, root, id)?;
            if head.revision != expected {
                return Err(Error::Conflict);
            }
            let mut charged = quota(&table, root)?;
            write(&mut table, root, id, head, value, &mut charged)?;
            head.revision = head.revision.checked_add(1).ok_or(Error::Capacity)?;
            retain(
                &mut table,
                &event_key(root, id, head.revision),
                &head.encode(),
                &mut charged,
            )?;
            engine(table.insert(head_key(root, id).as_slice(), head.encode().as_slice()))?;
            set_quota(&mut table, root, charged)?;
            object = Object { id, head };
        }
        engine(tx.commit())?;
        Ok(object)
    }
    /// Resolve from an explicit base in one read transaction; no `..`, global
    /// root, symlinks or source-copy fallback. `.` denotes the supplied base.
    pub fn lookup(&self, root: u8, base: u64, path: &[u8]) -> Result<u64> {
        if path.is_empty() || path.len() > metadata::PATH_BYTES || path.starts_with(b"/") {
            return Err(Error::Invalid);
        }
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        check_parent(&table, root, base)?;
        let mut id = base;
        for part in path.split(|b| *b == b'/') {
            if part == b"." {
                continue;
            }
            if part.is_empty() || part == b".." {
                return Err(Error::Invalid);
            }
            let name = Metadata::new(part, b"").ok_or(Error::Invalid)?.name;
            let record =
                engine(table.get(name_key(root, id, &name).as_slice()))?.ok_or(Error::Missing)?;
            id = u64::from_be_bytes(record.value().try_into().map_err(|_| Error::Corrupt)?);
        }
        Ok(id)
    }
    /// Exact media-type index range for direct children. Work is proportional
    /// to returned index entries, not the number or size of unrelated bodies.
    pub fn filter(
        &self,
        root: u8,
        parent: u64,
        media_type: &[u8],
        after: u64,
        out: &mut [u64],
    ) -> Result<usize> {
        let kind = metadata::field(media_type)
            .filter(|v| *v != [0; FIELD])
            .ok_or(Error::Invalid)?;
        let tx = engine(self.db.begin_read())?;
        let table = engine(tx.open_table(TABLE))?;
        check_parent(&table, root, parent)?;
        let start = type_key(
            root,
            parent,
            &kind,
            after.checked_add(1).ok_or(Error::Invalid)?,
        );
        let mut count = 0;
        for row in engine(table.range(start.as_slice()..))? {
            if count == out.len() {
                break;
            }
            let (k, _) = engine(row)?;
            let k = k.value();
            if k.len() != start.len() || k[..start.len() - 8] != start[..start.len() - 8] {
                break;
            }
            out[count] = u64::from_be_bytes(k[k.len() - 8..].try_into().unwrap());
            count += 1;
        }
        Ok(count)
    }
}
