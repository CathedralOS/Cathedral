//! One distribution migration: refresh stock copied launchers and label old
//! Notes/Colors, optionally refreshing stock app code for app-owned close UI.
//! Match retained template content,
//! never infer executable identity from a title or position alone.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use seed::{event, head, key, retain};
use std::collections::BTreeMap;
use std::io::Write;

type Records = BTreeMap<Vec<u8>, Vec<u8>>;

pub fn upgrade(
    path: &Path,
    offset: u64,
    bytes: u64,
    legacy: &[(u64, [u8; 32])],
    refresh_apps: bool,
) -> Result<()> {
    upgrade_with_quota(path, offset, bytes, legacy, refresh_apps, true)
}

/// The coordinated installer owns this scratch image and must finish the named
/// program replacements and enforce quota before publishing it. Collecting here
/// would judge a mixed generation and retire history needed by later migrations.
pub(super) fn stage(
    path: &Path,
    offset: u64,
    bytes: u64,
    legacy: &[(u64, [u8; 32])],
    refresh_apps: bool,
) -> Result<()> {
    upgrade_with_quota(path, offset, bytes, legacy, refresh_apps, false)
}

fn upgrade_with_quota(
    path: &Path,
    offset: u64,
    bytes: u64,
    legacy: &[(u64, [u8; 32])],
    refresh_apps: bool,
    enforce_quota: bool,
) -> Result<()> {
    let image = Arc::new(Image::open(path)?); // Excludes QEMU/another installer.
    let mut backup = vec![0; usize::try_from(image.len()?)?];
    image.read(0, &mut backup)?;
    let db = open(Region::new(image, offset, bytes)?)?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    let mut table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let mut records = Records::new();
    for item in table.iter()? {
        let (k, v) = item?;
        records.insert(k.value().to_vec(), v.value().to_vec());
    }
    let replacements = plan(&records, legacy, refresh_apps)?;
    let mut charged = u64::from_le_bytes(
        records
            .get(&key(b'q', &[]))
            .ok_or("missing quota")?
            .as_slice()
            .try_into()?,
    );
    if replacements.is_empty() && charged <= cathedral_database_objects::ROOT_QUOTA {
        println!("LABELS: no stock copies need upgrading");
        return Ok(());
    }
    // Preserve a recoverable pre-migration image before publishing any changes.
    let backup_path = path.with_file_name(format!(
        "{}.before-{}.raw",
        path.file_name()
            .ok_or("missing image name")?
            .to_string_lossy(),
        if refresh_apps {
            "app-close"
        } else {
            "readable-labels"
        }
    ));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup_path)
    {
        Ok(mut file) => {
            if let Err(error) = file.write_all(&backup).and_then(|_| file.sync_all()) {
                drop(file);
                // This invocation created the incomplete file; never reuse it.
                let _ = std::fs::remove_file(&backup_path);
                return Err(error.into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = std::fs::symlink_metadata(&backup_path)?;
            if !metadata.is_file() || metadata.len() != backup.len() as u64 {
                return Err("existing label backup is not a complete regular image".into());
            }
        }
        Err(error) => return Err(error.into()),
    }
    let mut store = Store::new(Policy {
        chunk_bytes: cathedral_database_objects::CHUNK_BYTES,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    for (object, content) in &replacements {
        let previous = &records[&key(b'h', &object.to_be_bytes())];
        let version = store.put(content);
        let publication = store.records(&version);
        for (k, v) in &publication {
            if k != b"head" {
                retain(&mut table, &mut charged, &key(k[0], &k[1..]), v)?;
                if k[0] == b'c' {
                    retain(&mut table, &mut charged, &key(b'k', &k[1..]), &[])?;
                }
            }
        }
        let revision = u64::from_le_bytes(previous[..8].try_into()?)
            .checked_add(1)
            .ok_or("revision overflow")?;
        let parent = u64::from_le_bytes(previous[48..].try_into()?);
        let next = head(
            revision,
            &publication[b"head".as_slice()],
            content.len(),
            parent,
        );
        retain(&mut table, &mut charged, &event(*object, revision), &next)?;
        table.insert(key(b'h', &object.to_be_bytes()).as_slice(), next.as_slice())?;
    }
    table.insert(key(b'q', &[]).as_slice(), charged.to_le_bytes().as_slice())?;
    // Identify stock copies before retiring old template events. Reclaim only
    // under pressure, after every replacement head has been staged, using the
    // guest's head/pin reachability policy inside this same transaction.
    let before_collection = charged;
    while enforce_quota && charged > cathedral_database_objects::ROOT_QUOTA {
        let pass = cathedral_database_objects::collect_records(&mut table, 0)
            .map_err(|error| format!("upgrade collection failed: {error:?}"))?;
        charged = pass.charged;
        if pass.removed == 0 {
            return Err(
                "live/pinned content exceeds root quota; no upgrade changes published".into(),
            );
        }
    }
    drop(table);
    tx.commit()?;
    if charged != before_collection {
        println!(
            "LABELS: collected unreachable revisions; charged bytes {before_collection} -> {charged}"
        );
    }
    println!(
        "LABELS: upgraded {} stock objects; saved data and custom programs preserved; backup {}",
        replacements.len(),
        backup_path.display()
    );
    Ok(())
}

fn plan(
    records: &Records,
    legacy: &[(u64, [u8; 32])],
    refresh_apps: bool,
) -> Result<Vec<(u64, Vec<u8>)>> {
    let mut heads = BTreeMap::new();
    for (k, v) in records {
        if k.starts_with(&[0, b'h']) {
            if k.len() != 10 || v.len() != 56 {
                return Err("invalid object head".into());
            }
            heads.insert(u64::from_be_bytes(k[2..].try_into()?), v);
        }
    }
    let latest = heads.get(&4).ok_or("missing template launcher")?;
    let launcher = content(records, latest)?;
    if !launcher.ends_with(&title(b"LAUNCHER")) {
        return Err("install the titled launcher before upgrading users".into());
    }
    let mut replacements = Vec::new();
    for (&object, current) in &heads {
        let parent = u64::from_le_bytes(current[48..].try_into()?);
        if refresh_apps && parent == 2 && object > 3 {
            let known = legacy
                .iter()
                .any(|(id, digest)| *id == 3 && digest[..] == current[8..40])
                || records.iter().any(|(k, v)| {
                    k.len() == 18
                        && k.starts_with(&key(b'e', &3u64.to_be_bytes()))
                        && v.len() == 56
                        && v[8..40] == current[8..40]
                });
            if known {
                let next = content(records, heads.get(&3).ok_or("missing template matrix")?)?;
                if content(records, current)? != next {
                    replacements.push((object, next));
                }
            }
        }
        let Some(user) = heads.get(&parent) else {
            continue;
        };
        if parent <= 3 || u64::from_le_bytes(user[48..].try_into()?) != 2 {
            continue;
        }
        for (template, label) in [
            (4u64, b"LAUNCHER".as_slice()),
            (5, b"NOTES"),
            (6, b"COLORS"),
        ] {
            let known = |digest: &[u8]| {
                legacy
                    .iter()
                    .any(|(id, saved)| *id == template && saved[..] == *digest)
                    || records.iter().any(|(k, v)| {
                        k.len() == 18
                            && k.starts_with(&key(b'e', &template.to_be_bytes()))
                            && v.len() == 56
                            && v[8..40] == *digest
                    })
            };
            let matches = known(&current[8..40])
                || (refresh_apps
                    && content(records, current).ok().is_some_and(|old| {
                        // The previous label migration appended only this footer. Match
                        // its original code, not the user-visible title by itself.
                        old.ends_with(&title(label)) && known(&digest(&old[..old.len() - 32]))
                    }));
            if !matches {
                continue;
            }
            let old = content(records, current)?;
            let new = if refresh_apps {
                let target = content(records, heads.get(&template).ok_or("missing stock app")?)?;
                if !target.ends_with(&title(label)) {
                    return Err("stock app title mismatch".into());
                }
                target
            } else if template == 4 {
                launcher.clone()
            } else if old.ends_with(&title(label)) {
                old.clone()
            } else {
                let mut named = old.clone();
                named.extend_from_slice(&title(label));
                named
            };
            if new.len() > cathedral_database_objects::MAX_BYTES {
                return Err("titled program exceeds guest object limit".into());
            }
            if old != new {
                replacements.push((object, new));
            }
            break;
        }
    }
    Ok(replacements)
}

fn digest(data: &[u8]) -> Vec<u8> {
    let mut store = Store::new(Policy {
        chunk_bytes: cathedral_database_objects::CHUNK_BYTES,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(data);
    store.records(&version)[b"head".as_slice()].clone()
}

fn title(text: &[u8]) -> [u8; 32] {
    let mut footer = [0; 32];
    footer[..8].copy_from_slice(b"CTHTIT01");
    footer[8..8 + text.len()].copy_from_slice(text);
    footer
}

fn content(records: &Records, head: &[u8]) -> Result<Vec<u8>> {
    let length = usize::try_from(u64::from_le_bytes(head[40..48].try_into()?))?;
    if length > cathedral_database_objects::MAX_BYTES {
        return Err("oversized program".into());
    }
    let manifest = records
        .get(&key(b'm', &head[8..40]))
        .ok_or("missing manifest")?;
    if manifest.len() < 17 {
        return Err("short manifest".into());
    }
    let mut data = Vec::new();
    match manifest[0] {
        0 => data.extend_from_slice(&manifest[17..]),
        1 if (manifest.len() - 17).is_multiple_of(32) => {
            for digest in manifest[17..].chunks_exact(32) {
                data.extend_from_slice(records.get(&key(b'c', digest)).ok_or("missing chunk")?);
            }
        }
        _ => return Err("invalid manifest".into()),
    }
    if data.len() != length || !data.starts_with(b"\x7fELF") {
        return Err("not a complete program".into());
    }
    // Independently reconstruct the digest before identifying or replacing code.
    let mut store = Store::new(Policy {
        chunk_bytes: cathedral_database_objects::CHUNK_BYTES,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(&data);
    if store.records(&version)[b"head".as_slice()] != head[8..40] {
        return Err("program digest mismatch".into());
    }
    Ok(data)
}

/// Distribution-owned allowlist for stock revisions already swept from history.
/// Lines are template object (3..6) and the content-layout manifest digest.
pub fn legacy(path: &Path) -> Result<Vec<(u64, [u8; 32])>> {
    let mut entries = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.len() != 2 || words[1].len() != 64 || !words[1].is_ascii() {
            return Err("invalid legacy title entry".into());
        }
        let object = words[0].parse()?;
        if !(3..=6).contains(&object) {
            return Err(
                "legacy titles may identify only stock matrix/launcher/Notes/Colors".into(),
            );
        }
        let mut digest = [0; 32];
        for (i, byte) in digest.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&words[1][i * 2..i * 2 + 2], 16)?;
        }
        entries.push((object, digest));
    }
    Ok(entries)
}

#[cfg(test)]
#[path = "labels_tests.rs"]
mod tests;
