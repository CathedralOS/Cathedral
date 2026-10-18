//! Offline installer maintenance: back up the locked image, compact physical
//! pages, then cold-open and compare every record. No history is collected.
use super::*;
use redb::{ReadableTable, TableHandle};
use std::{collections::BTreeMap, io::Write};

type Records = BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>;

fn records(db: &Database) -> Result<Records> {
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
    let image = Arc::new(Image::open(path)?); // Keep the exclusive lock through verification.
    let region = Region::new(image.clone(), offset, bytes)?;
    let header = region.headers()?;
    if !header.ready || header.length == 0 {
        return Err("cannot compact an incomplete volume".into());
    }
    let mut original = vec![0; usize::try_from(image.len()?)?];
    image.read(0, &mut original)?;
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
    let mut db = open(region.clone())?;
    let before = records(&db)?;
    db.compact()?;
    drop(db);
    let db = open(region.clone())?;
    if records(&db)? != before {
        return Err("compaction changed records; retain the backup and stop".into());
    }
    drop(db);
    let after = region.headers()?;
    if after.id != header.id || after.region_bytes != header.region_bytes {
        return Err("compaction changed volume identity or assigned extent".into());
    }
    println!(
        "COMPACTED: {} records preserved; engine bytes {} -> {}",
        before.values().map(BTreeMap::len).sum::<usize>(),
        header.length,
        after.length
    );
    Ok(())
}

#[cfg(test)]
#[path = "compact_tests.rs"]
mod tests;
