//! Host fixture preparation for the guest path/configuration proof. Call only on
//! a disposable image copy. No program bytes or application state are replaced.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use cathedral_desktop_matrix_contracts::matrix;
use cathedral_platform_contracts::storage::metadata::{self, Metadata};

pub fn prepare(path: &Path, offset: u64, bytes: u64, missing: bool) -> Result<()> {
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
                .ok_or("missing named entry")?
                .as_slice()
                .try_into()?,
        ))
    };
    let users = named(0, b"Users")?;
    let user = named(users, b"User 1")?;
    let launcher = named(user, b"Desktop")?;
    let config = named(user, matrix::config::NAME)?;
    let mut suffix = user.to_be_bytes().to_vec();
    suffix.extend_from_slice(&launcher.to_be_bytes());
    let edge = seed::key(b't', &suffix);
    let mut m = Metadata::decode(&rows[&edge]).ok_or("metadata")?;
    let mut old = user.to_be_bytes().to_vec();
    old.extend_from_slice(&m.name);
    rows.remove(&seed::key(b'd', &old));
    table.remove(seed::key(b'd', &old).as_slice())?;
    m.name = metadata::field(b"Workspace").unwrap();
    let mut new = user.to_be_bytes().to_vec();
    new.extend_from_slice(&m.name);
    rows.insert(seed::key(b'd', &new), launcher.to_be_bytes().to_vec());
    rows.insert(edge, m.encode().to_vec());
    let mut saved = matrix::config::Config::decode(&hierarchy::configuration()).unwrap();
    saved.launcher = metadata::field(if missing { b"Missing" } else { b"./Workspace" }).unwrap();
    let mut content = Store::new(Policy {
        chunk_bytes: cathedral_database_objects::CHUNK_BYTES,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = content.put(&saved.encode());
    let records = content.records(&version);
    let revision =
        u64::from_le_bytes(rows[&seed::key(b'h', &config.to_be_bytes())][..8].try_into()?) + 1;
    let head = seed::head(
        revision,
        &records[b"head".as_slice()],
        matrix::config::SIZE,
        user,
    );
    for (k, v) in records {
        if k != b"head" {
            rows.insert(seed::key(k[0], &k[1..]), v);
        }
    }
    rows.insert(seed::key(b'h', &config.to_be_bytes()), head.clone());
    rows.insert(seed::event(config, revision), head);
    // An extension-looking text entry must not become a program tile.
    let id = u64::from_le_bytes(rows[&seed::key(b'n', &[])].as_slice().try_into()?);
    let version = content.put(b"This is a document, not a program.");
    let records = content.records(&version);
    let head = seed::head(
        1,
        &records[b"head".as_slice()],
        b"This is a document, not a program.".len(),
        user,
    );
    for (k, v) in records {
        if k != b"head" {
            rows.insert(seed::key(k[0], &k[1..]), v);
        }
    }
    rows.insert(seed::key(b'h', &id.to_be_bytes()), head.clone());
    rows.insert(seed::event(id, 1), head);
    let mut e = user.to_be_bytes().to_vec();
    e.extend_from_slice(&id.to_be_bytes());
    rows.insert(seed::key(b't', &e), Vec::new());
    hierarchy::describe(&mut rows, id, b"Fake.exe", metadata::TEXT)?;
    rows.insert(seed::key(b'n', &[]), (id + 1).to_le_bytes().to_vec());
    let charge = rows
        .iter()
        .filter(|(k, _)| k[0] == 0)
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>();
    rows.insert(seed::key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    for (k, v) in rows {
        table.insert(k.as_slice(), v.as_slice())?;
    }
    drop(table);
    tx.commit()?;
    Ok(())
}
