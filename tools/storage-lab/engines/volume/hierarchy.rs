//! Explicit stock-desktop migration. Add hierarchy metadata and each matrix's
//! saved arrangement atomically; existing names/configuration are never replaced.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use cathedral_desktop_matrix_contracts::matrix;
use cathedral_platform_contracts::storage::metadata::{self, Metadata};
use redb::ReadableTable;
use std::collections::BTreeMap;
type Records = BTreeMap<Vec<u8>, Vec<u8>>;

pub fn configuration() -> Vec<u8> {
    // The host encodes the distribution's defaults; it does not select policy.
    let fields: BTreeMap<_, _> = include_str!("../../../../source-rs/distribution/matrix.conf")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split_once('=').expect("matrix.conf key=value"))
        .collect();
    assert_eq!(fields.len(), 9, "matrix.conf fields");
    let number = |key| fields[key].parse::<u64>().expect("matrix.conf integer");
    assert_eq!(fields["keys"], "all", "matrix.conf key selection");
    let config = matrix::config::Config {
        launcher: metadata::field(fields["launcher"].as_bytes()).expect("matrix.conf launcher"),
        launcher_pages: number("launcher_pages"),
        app_pages: number("app_pages"),
        app_readers: number("app_readers"),
        margin: number("margin"),
        launcher_height: number("launcher_height"),
        gap: number("gap"),
        app_surfaces: number("app_surfaces"),
        keys: matrix::VALID_KEY_MASK,
    };
    assert_eq!(
        matrix::config::Config::decode(&config.encode()),
        Some(config),
        "invalid matrix.conf"
    );
    config.encode().to_vec()
}
fn edge(parent: u64, id: u64) -> Vec<u8> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&id.to_be_bytes());
    seed::key(b't', &suffix)
}
pub(super) fn describe(records: &mut Records, id: u64, name: &[u8], kind: &[u8]) -> Result<()> {
    let head = records
        .get(&seed::key(b'h', &id.to_be_bytes()))
        .ok_or("missing metadata object")?;
    let parent = u64::from_le_bytes(head[48..].try_into()?);
    let k = edge(parent, id);
    let existing = records.get(&k).ok_or("missing hierarchy edge")?;
    if !existing.is_empty() {
        return Ok(());
    }
    let m = Metadata::new(name, kind).ok_or("invalid metadata")?;
    records.insert(k, m.encode().to_vec());
    if !name.is_empty() {
        let mut suffix = parent.to_be_bytes().to_vec();
        suffix.extend_from_slice(&m.name);
        let k = seed::key(b'd', &suffix);
        if records.contains_key(&k) {
            return Err("name collision; migration refused".into());
        }
        records.insert(k, id.to_be_bytes().to_vec());
    }
    if !kind.is_empty() {
        let mut suffix = parent.to_be_bytes().to_vec();
        suffix.extend_from_slice(&m.media_type);
        suffix.extend_from_slice(&id.to_be_bytes());
        records.insert(seed::key(b'y', &suffix), Vec::new());
    }
    Ok(())
}
pub(super) fn insert_config(records: &mut Records, id: u64, parent: u64) -> Result<()> {
    let bytes = configuration();
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(&bytes);
    let content = store.records(&version);
    let head = seed::head(1, &content[b"head".as_slice()], bytes.len(), parent);
    for (k, v) in content {
        if k != b"head" {
            records.insert(seed::key(k[0], &k[1..]), v);
        }
    }
    records.insert(seed::key(b'h', &id.to_be_bytes()), head.clone());
    records.insert(seed::event(id, 1), head);
    records.insert(edge(parent, id), Vec::new());
    describe(records, id, matrix::config::NAME, metadata::MATRIX_CONFIG)
}
pub fn upgrade(path: &Path, offset: u64, bytes: u64, additions: &[String]) -> Result<()> {
    let image = Arc::new(Image::open(path)?);
    let mut backup = vec![0; usize::try_from(image.len()?)?];
    image.read(0, &mut backup)?;
    let db = open(Region::new(image, offset, bytes)?)?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    let mut table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let mut records = Records::new();
    for row in table.iter()? {
        let (k, v) = row?;
        records.insert(k.value().to_vec(), v.value().to_vec());
    }
    let before = records.clone();
    plan(&mut records)?;
    realm_programs::install(&mut records, additions)?;
    let mut charged = u64::from_le_bytes(records[&seed::key(b'q', &[])].as_slice().try_into()?);
    if records == before && charged <= cathedral_database_objects::ROOT_QUOTA {
        println!("HIERARCHY: already installed");
        return Ok(());
    }
    // Save the exclusively opened image before the single migration commit.
    let backup_path = path.with_file_name(format!(
        "{}.before-{}.raw",
        path.file_name().ok_or("image name")?.to_string_lossy(),
        if additions.is_empty() {
            "hierarchy"
        } else {
            "realm-programs"
        }
    ));
    use std::io::Write;
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup_path)
    {
        Ok(mut file) => {
            file.write_all(&backup)?;
            file.sync_all()?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let m = std::fs::symlink_metadata(&backup_path)?;
            if !m.is_file() || m.len() != backup.len() as u64 {
                return Err("invalid existing hierarchy backup".into());
            }
        }
        Err(e) => return Err(e.into()),
    }
    for (k, v) in &records {
        if before.get(k) != Some(v) {
            table.insert(k.as_slice(), v.as_slice())?;
        }
    }
    let before_collection = charged;
    while charged > cathedral_database_objects::ROOT_QUOTA {
        let pass = cathedral_database_objects::collect_records(&mut table, 0)
            .map_err(|error| format!("hierarchy collection failed: {error:?}"))?;
        charged = pass.charged;
        if pass.removed == 0 {
            return Err(format!(
                "live/pinned realm programs need {charged} bytes, exceeding root quota {}; no upgrade changes published",
                cathedral_database_objects::ROOT_QUOTA
            ).into());
        }
    }
    drop(table);
    tx.commit()?;
    if charged != before_collection {
        println!(
            "HIERARCHY: collected unreachable revisions; charged bytes {before_collection} -> {charged}"
        );
    }
    println!(
        "HIERARCHY: installed names, media indexes and matrix configuration; backup {}",
        backup_path.display()
    );
    Ok(())
}
fn plan(records: &mut Records) -> Result<()> {
    // Only the explicitly selected six-object stock desktop owns these roots.
    for (id, name) in [
        (1u64, b"Login".as_slice()),
        (2, b"Users"),
        (3, b"Template"),
        (4, b"Desktop"),
        (5, b"Notes"),
        (6, b"Colors"),
    ] {
        describe(
            records,
            id,
            name,
            if id == 2 { b"" } else { metadata::EXECUTABLE },
        )?;
    }
    let mut users = vec![3u64];
    let heads: Vec<(u64, Vec<u8>)> = records
        .iter()
        .filter(|(k, _)| k.len() == 10 && k.starts_with(&[0, b'h']))
        .map(|(k, v)| (u64::from_be_bytes(k[2..].try_into().unwrap()), v.clone()))
        .collect();
    for (id, head) in &heads {
        if *id > 3 && u64::from_le_bytes(head[48..].try_into()?) == 2 {
            // Identify the executable by retained template history, never by position.
            let known = records.iter().any(|(k, v)| {
                k.starts_with(&seed::key(b'e', &3u64.to_be_bytes()))
                    && v.len() == 56
                    && v[8..40] == head[8..40]
            });
            if known {
                describe(
                    records,
                    *id,
                    format!("User {}", users.len()).as_bytes(),
                    metadata::EXECUTABLE,
                )?;
                users.push(*id);
            }
        }
    }
    for &user in &users {
        for (id, head) in &heads {
            if u64::from_le_bytes(head[48..].try_into()?) != user {
                continue;
            }
            for (source, name) in [(4u64, b"Desktop".as_slice()), (5, b"Notes"), (6, b"Colors")] {
                let known = records.iter().any(|(k, v)| {
                    k.starts_with(&seed::key(b'e', &source.to_be_bytes()))
                        && v.len() == 56
                        && v[8..40] == head[8..40]
                });
                if known {
                    describe(records, *id, name, metadata::EXECUTABLE)?;
                    break;
                }
            }
        }
        let present = heads.iter().any(|(id, head)| {
            u64::from_le_bytes(head[48..].try_into().unwrap()) == user
                && records
                    .get(&edge(user, *id))
                    .and_then(|v| Metadata::decode(v))
                    .is_some_and(|m| metadata::text(&m.name) == matrix::config::NAME)
        });
        if !present {
            let id = u64::from_le_bytes(
                records
                    .get(&seed::key(b'n', &[]))
                    .ok_or("next number")?
                    .as_slice()
                    .try_into()?,
            );
            insert_config(records, id, user)?;
            records.insert(
                seed::key(b'n', &[]),
                id.checked_add(1)
                    .ok_or("object overflow")?
                    .to_le_bytes()
                    .to_vec(),
            );
        }
    }
    let charged = records
        .iter()
        .filter(|(k, _)| k[0] == 0)
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>();
    // Admission follows all named replacements and reachability collection in
    // upgrade(); intermediate old/new generations may exceed the final quota.
    records.insert(
        seed::key(b'q', &[]),
        (charged as u64).to_le_bytes().to_vec(),
    );
    Ok(())
}

pub fn inspect(path: &Path, offset: u64, bytes: u64) -> Result<()> {
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    for row in table.iter()? {
        let (k, v) = row?;
        let k = k.value();
        let v = v.value();
        if k.len() == 10 && k.starts_with(&[0, b'h']) && v.len() == 56 {
            let id = u64::from_be_bytes(k[2..].try_into()?);
            let parent = u64::from_le_bytes(v[48..].try_into()?);
            let digest = v[8..40]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let entry = table.get(edge(parent, id).as_slice())?;
            let metadata = entry.as_ref().and_then(|v| Metadata::decode(v.value()));
            let name = metadata
                .as_ref()
                .map(|m| String::from_utf8_lossy(metadata::text(&m.name)));
            println!("OBJECT {id} parent={parent} manifest={digest} name={name:?}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_preserves_existing_state_and_is_idempotent() {
        let programs = [
            (0, b"elf login".as_slice()),
            (0, b""),
            (2, b"elf matrix"),
            (3, b"elf launcher"),
            (3, b"elf notes"),
            (3, b"elf colors"),
        ]
        .map(|(parent, bytes)| seed::Program {
            parent,
            bytes: bytes.to_vec(),
        });
        let mut rows = seed::expected(&programs);
        for source in 3u64..=6 {
            let copy = source + 4;
            let parent = if source == 3 { 2u64 } else { 7 };
            let mut head = rows[&seed::key(b'h', &source.to_be_bytes())].clone();
            head[48..].copy_from_slice(&parent.to_le_bytes());
            rows.insert(seed::key(b'h', &copy.to_be_bytes()), head.clone());
            rows.insert(seed::event(copy, 1), head);
            rows.insert(edge(parent, copy), vec![]);
        }
        // Unrelated retained state must survive byte-for-byte, even when unnamed.
        rows.insert(vec![1, b'z'], b"other root untouched".to_vec());
        rows.insert(seed::key(b'n', &[]), 11u64.to_le_bytes().to_vec());
        rows.insert(seed::key(b'q', &[]), 0u64.to_le_bytes().to_vec());
        let original = rows.clone();
        plan(&mut rows).unwrap();
        for (k, v) in original
            .iter()
            .filter(|(k, _)| !matches!(k[1], b't' | b'n' | b'q'))
        {
            assert_eq!(rows.get(k), Some(v));
        }
        assert_eq!(
            Metadata::decode(&rows[&edge(7, 9)]).unwrap().name,
            metadata::field(b"Notes").unwrap()
        );
        assert_eq!(
            u64::from_le_bytes(rows[&seed::key(b'n', &[])].as_slice().try_into().unwrap()),
            13
        );
        let once = rows.clone();
        plan(&mut rows).unwrap();
        assert_eq!(rows, once);
        let charged = rows
            .iter()
            .filter(|(k, _)| k[0] == 0)
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();
        assert_eq!(rows[&seed::key(b'q', &[])], (charged as u64).to_le_bytes());
    }
}
