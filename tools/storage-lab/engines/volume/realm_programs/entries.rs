//! Resolve local paths and publish named program revisions without changing identities.
use super::*;
pub(super) fn named(rows: &Records, parent: u64, name: &[u8]) -> Result<Option<u64>> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&metadata::field(name).ok_or("invalid program name")?);
    let Some(value) = rows.get(&seed::key(b'd', &suffix)) else {
        return Ok(None);
    };
    let id = u64::from_be_bytes(value.as_slice().try_into()?);
    let head = rows
        .get(&seed::key(b'h', &id.to_be_bytes()))
        .ok_or("name points to missing object")?;
    if head.len() != 56 || u64::from_le_bytes(head[48..].try_into()?) != parent {
        return Err("name/head parent mismatch".into());
    }
    Ok(Some(id))
}
pub(super) fn resolve(rows: &Records, root: u64, path: &str) -> Result<Option<u64>> {
    let mut parent = root;
    for name in path.split('/') {
        if name.is_empty() || matches!(name, "." | "..") {
            return Err("invalid install path".into());
        }
        let Some(id) = named(rows, parent, name.as_bytes())? else {
            return Ok(None);
        };
        parent = id;
    }
    Ok(Some(parent))
}
pub(super) fn put(rows: &mut Records, id: u64, parent: u64, data: &[u8]) -> Result<()> {
    let key = seed::key(b'h', &id.to_be_bytes());
    if let Some(head) = rows.get(&key)
        && (head.len() != 56 || u64::from_le_bytes(head[48..].try_into()?) != parent)
    {
        return Err("refusing to move an existing object during install".into());
    }
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(data);
    let records = store.records(&version);
    let digest = &records[b"head".as_slice()];
    if rows.get(&key).is_some_and(|h| h[8..40] == digest[..]) {
        return Ok(());
    }
    let revision = rows.get(&key).map_or(Ok(1), |h| {
        u64::from_le_bytes(h[..8].try_into().unwrap())
            .checked_add(1)
            .ok_or("revision overflow")
    })?;
    for (k, v) in &records {
        if k != b"head" {
            rows.insert(seed::key(k[0], &k[1..]), v.clone());
            if k[0] == b'c' {
                rows.insert(seed::key(b'k', &k[1..]), vec![]);
            }
        }
    }
    let head = seed::head(revision, digest, data.len(), parent);
    rows.insert(key, head.clone());
    rows.insert(seed::event(id, revision), head);
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&id.to_be_bytes());
    rows.entry(seed::key(b't', &suffix)).or_default();
    Ok(())
}
