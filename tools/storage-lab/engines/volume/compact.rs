//! Offline installer maintenance: back up the locked image, rebuild its records
//! in host scratch space, then capacity-check, publish and cold-verify. No history is collected.
//! Rebuilding off-image avoids the spare pages required by in-place compaction.
use super::*;
use redb::{ReadableTable, TableHandle};
use std::{collections::BTreeMap, io::Write};

pub(super) type Records = BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>;

pub(super) fn records(db: &Database) -> Result<Records> {
    let tx = db.begin_read()?;
    if tx.list_multimap_tables()?.next().is_some() {
        return Err("installer compaction does not support multimap tables".into());
    }
    let mut result = Records::new();
    for handle in tx.list_tables()? {
        let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new(handle.name()))?;
        let mut rows = BTreeMap::new();
        for row in table.iter()? {
            let (key, value) = row?;
            rows.insert(key.value().to_vec(), value.value().to_vec());
        }
        result.insert(handle.name().to_owned(), rows);
    }
    Ok(result)
}

pub fn run(path: &Path, offset: u64, bytes: u64) -> Result<()> {
    rewrite(path, offset, bytes, |records, _| Ok(records))
}

/// Explicit offline growth keeps the volume identity, every record and both
/// outside guards. The original image is backed up before its extent changes.
pub fn grow(path: &Path, offset: u64, bytes: u64, target: u64) -> Result<()> {
    if target < bytes {
        return Err("volume growth cannot shrink the assigned extent".into());
    }
    rewrite_extent(path, offset, bytes, target, |records, _| Ok(records))
}

/// Keep the source locked and unchanged while a host-only edit prepares its result.
pub(super) fn rewrite(
    path: &Path,
    offset: u64,
    bytes: u64,
    edit: impl FnOnce(Records, &Header) -> Result<Records>,
) -> Result<()> {
    rewrite_extent(path, offset, bytes, bytes, edit)
}

fn rewrite_extent(
    path: &Path,
    offset: u64,
    bytes: u64,
    target: u64,
    edit: impl FnOnce(Records, &Header) -> Result<Records>,
) -> Result<()> {
    geometry(target)?;
    let image = Arc::new(Image::open(path)?); // Keep the exclusive lock through verification.
    let region = Region::new(image.clone(), offset, bytes)?;
    let header = region.headers()?;
    if !header.ready || header.length == 0 {
        return Err("cannot compact an incomplete volume".into());
    }
    let mut original = vec![0; usize::try_from(image.len()?)?];
    image.read(0, &mut original)?;
    let target_len = (original.len() as u64)
        .checked_add(target - bytes)
        .ok_or("image growth overflow")?;
    let suffix = &original[usize::try_from(offset + bytes)?..];
    // A content-named backup never overwrites an earlier recovery point, and
    // repeated attempts on identical input do not accumulate duplicate backups.
    let backup = path.with_file_name(format!(
        "{}.before-compact-{}.raw",
        path.file_name()
            .ok_or("missing image name")?
            .to_string_lossy(),
        blake3::hash(&original).to_hex()
    ));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
    {
        Ok(mut file) => {
            if let Err(error) = file.write_all(&original).and_then(|_| file.sync_all()) {
                drop(file);
                let _ = std::fs::remove_file(&backup);
                return Err(error.into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if !std::fs::symlink_metadata(&backup)?.is_file() || std::fs::read(&backup)? != original
            {
                return Err("existing compaction backup differs; refusing maintenance".into());
            }
        }
        Err(error) => return Err(error.into()),
    }
    println!("COMPACT: backup {}", backup.display());
    // Even opening redb can write recovery metadata. Work entirely on a copy
    // until the rebuilt, capacity-checked volume has passed a cold comparison.
    let source = Arc::new(redb::backends::InMemoryBackend::new());
    source.set_len(bytes)?;
    source.write(0, &original[offset as usize..(offset + bytes) as usize])?;
    let db = open(Region::new(source, 0, bytes)?)
        .map_err(|e| format!("opening source copy for maintenance: {e}"))?;
    let before = records(&db)?;
    drop(db);
    let before = edit(before, &header)?;
    let mut target_header = header.clone();
    target_header.region_bytes = target;
    let rebuilt = rebuild(&before, &target_header).map_err(|e| {
        format!(
            "rebuilding {} records: {e}",
            before.values().map(BTreeMap::len).sum::<usize>()
        )
    })?;
    let publish = (|| -> Result<_> {
        if target != bytes {
            image.resize_for_maintenance(target_len)?;
            image.write(offset + target, suffix)?;
        }
        let region = Region::new(image.clone(), offset, target)?;
        region.write(0, &rebuilt)?;
        region.sync()?;
        let db = open(region.clone())?;
        if records(&db)? != before {
            return Err("rebuilt volume changed records".into());
        }
        drop(db);
        let after = region.headers()?;
        if after.id != header.id || after.region_bytes != target {
            return Err("rebuild changed volume identity or assigned extent".into());
        }
        let mut guards = vec![0; suffix.len()];
        image.read(offset + target, &mut guards)?;
        if guards != suffix {
            return Err("rebuild changed trailing guard bytes".into());
        }
        Ok(after)
    })();
    let after = match publish {
        Ok(after) => after,
        Err(error) => {
            // Ordinary I/O failures restore the input. A process/power loss during
            // publication instead requires recovery from the synced backup.
            image.resize_for_maintenance(original.len() as u64)?;
            image.write(0, &original)?;
            image.sync_data()?;
            return Err(error);
        }
    };
    println!(
        "VERIFIED: {} prepared records; engine bytes {} -> {}",
        before.values().map(BTreeMap::len).sum::<usize>(),
        header.length,
        after.length
    );
    Ok(())
}

/// Rebuild in bounded host scratch space. Only a result fitting the original
/// extent can be published; every table, including history, survives.
pub(super) fn rebuild(before: &Records, header: &Header) -> Result<Vec<u8>> {
    let memory = Arc::new(redb::backends::InMemoryBackend::new());
    let scratch_bytes = header
        .region_bytes
        .checked_mul(4)
        .ok_or("scratch overflow")?;
    memory.set_len(scratch_bytes)?;
    let region = Region::new(memory.clone(), 0, scratch_bytes)?;
    format(region.clone(), header.id)?;
    let mut db = open(region.clone())?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    for (name, rows) in before {
        let mut table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new(name))?;
        for (key, value) in rows {
            table.insert(key.as_slice(), value.as_slice())?;
        }
    }
    tx.commit()?;
    db.compact()?;
    drop(db);
    let db = open(region.clone())?;
    if records(&db)? != *before {
        return Err("rebuilt volume differs before publication".into());
    }
    drop(db);
    // Continue the envelope generation rather than resetting an existing volume.
    let mut fresh = region.headers()?;
    if fresh.length > header.capacity() {
        return Err("compacted records do not fit the assigned volume".into());
    }
    fresh.region_bytes = header.region_bytes;
    fresh.generation = header
        .generation
        .checked_add(1)
        .ok_or("generation exhausted")?;
    let target = Region::new(memory.clone(), 0, header.region_bytes)?;
    target.publish_header(&fresh)?;
    let db = open(target)?;
    if records(&db)? != *before {
        return Err("rebuilt volume differs at original capacity".into());
    }
    drop(db);
    let mut result = vec![0; usize::try_from(header.region_bytes)?];
    memory.read(0, &mut result)?;
    Ok(result)
}

#[cfg(test)]
#[path = "compact_tests.rs"]
mod tests;
