//! Fault fixture only: replace one named installed program on a disposable
//! image, retaining its identity, metadata, subtree and all unrelated state.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use cathedral_platform_contracts::storage::metadata;

pub fn prepare(path: &Path, offset: u64, bytes: u64, target: &str, content: &[u8]) -> Result<()> {
    if content.len() > 256 * 1024 {
        return Err("fixture program too large".into());
    }
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    let mut table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let mut rows = colors::records(&table)?;
    let named = |parent: u64, name: &[u8]| -> Result<u64> {
        let mut suffix = parent.to_be_bytes().to_vec();
        suffix.extend_from_slice(&metadata::field(name).ok_or("name")?);
        Ok(u64::from_be_bytes(
            rows.get(&seed::key(b'd', &suffix))
                .ok_or("missing fixture path")?
                .as_slice()
                .try_into()?,
        ))
    };
    let id = if target == "login" {
        named(0, b"Login")?
    } else {
        let users = named(0, b"Users")?;
        let user = named(users, b"User 1")?;
        match target {
            "notes" => named(user, b"Notes")?,
            "launcher" => named(user, b"Desktop")?,
            "user" => user,
            _ => return Err("fixture target must be login, notes, launcher or user".into()),
        }
    };
    let head_key = seed::key(b'h', &id.to_be_bytes());
    let old = &rows[&head_key];
    let revision = u64::from_le_bytes(old[..8].try_into()?)
        .checked_add(1)
        .ok_or("revision overflow")?;
    let parent = u64::from_le_bytes(old[48..].try_into()?);
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(content);
    let records = store.records(&version);
    let head = seed::head(
        revision,
        &records[b"head".as_slice()],
        content.len(),
        parent,
    );
    for (key, value) in records {
        if key != b"head" {
            if key[0] == b'c' {
                rows.insert(seed::key(b'k', &key[1..]), Vec::new());
            }
            rows.insert(seed::key(key[0], &key[1..]), value);
        }
    }
    rows.insert(head_key, head.clone());
    rows.insert(seed::event(id, revision), head);
    let charge = rows
        .iter()
        .filter(|(key, _)| key[0] == 0)
        .map(|(key, value)| key.len() + value.len())
        .sum::<usize>();
    if charge > 1024 * 1024 {
        return Err("fixture exceeds root quota".into());
    }
    rows.insert(seed::key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    for (key, value) in rows {
        // Preserve unchanged pages. Rewriting every installed program for one
        // probe can exhaust the tiny volume's copy-on-write transaction space.
        let changed = table
            .get(key.as_slice())?
            .is_none_or(|old| old.value() != value.as_slice());
        if changed {
            table.insert(key.as_slice(), value.as_slice())?;
        }
    }
    drop(table);
    tx.commit()?;
    println!("FAILURE FIXTURE: replaced {target}, object {id}; other state retained");
    Ok(())
}

/// Hide a root path by a consistent hierarchy rename, not database corruption.
pub fn rename(path: &Path, offset: u64, bytes: u64, target: &str) -> Result<()> {
    let name = match target {
        "login" => b"Login".as_slice(),
        "users" => b"Users".as_slice(),
        _ => return Err("root path must be login or users".into()),
    };
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    let mut table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let mut rows = colors::records(&table)?;
    let mut old = 0u64.to_be_bytes().to_vec();
    old.extend_from_slice(&metadata::field(name).unwrap());
    let id = rows
        .remove(&seed::key(b'd', &old))
        .ok_or("missing root path")?;
    let mut edge = 0u64.to_be_bytes().to_vec();
    edge.extend_from_slice(&id);
    let key = seed::key(b't', &edge);
    let mut m = metadata::Metadata::decode(&rows[&key]).ok_or("metadata")?;
    m.name = metadata::field(b"Unavailable").unwrap();
    let mut new = 0u64.to_be_bytes().to_vec();
    new.extend_from_slice(&m.name);
    if rows.contains_key(&seed::key(b'd', &new)) {
        return Err("rename collision".into());
    }
    rows.insert(seed::key(b'd', &new), id);
    rows.insert(key, m.encode().to_vec());
    // Fixed-width names leave the accounted record size unchanged.
    table.remove(seed::key(b'd', &old).as_slice())?;
    for (key, value) in rows {
        table.insert(key.as_slice(), value.as_slice())?;
    }
    drop(table);
    tx.commit()?;
    println!("FAILURE FIXTURE: renamed root {target}; object contents retained");
    Ok(())
}
