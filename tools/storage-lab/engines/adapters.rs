//! Engine API adapters. Each batch is one durable transaction in one byte-key table.
use super::trace::{Batch, Records};
use super::{
    Result,
    backend::{Backend, Counts},
};
use redb::{Database, ReadableDatabase, TableDefinition};
use rusqlite::{Connection, params};
use std::path::Path;

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("records");

pub enum Engine {
    Redb(Database, Backend),
    Sqlite(Connection),
}

impl Engine {
    pub fn open(kind: &str, path: &Path) -> Result<Self> {
        match kind {
            "redb" => {
                let mut builder = Database::builder();
                builder.set_cache_size(8 * 1024 * 1024);
                let backend = Backend::open(path)?;
                let db = builder.create_with_backend(backend.clone())?;
                // Bootstrap only when the table is absent; reopening a populated
                // database must not silently add a transaction before inspection.
                let missing = match db.begin_read()?.open_table(TABLE) {
                    Ok(_) => false,
                    Err(redb::TableError::TableDoesNotExist(_)) => true,
                    Err(error) => return Err(error.into()),
                };
                if missing {
                    let mut tx = db.begin_write()?;
                    tx.set_durability(redb::Durability::Immediate)?;
                    tx.open_table(TABLE)?;
                    tx.commit()?;
                }
                Ok(Self::Redb(db, backend))
            }
            "sqlite_wal" | "sqlite_delete" => {
                let db = Connection::open(path)?;
                let journal = if kind == "sqlite_wal" {
                    "WAL"
                } else {
                    "DELETE"
                };
                db.execute_batch(&format!("PRAGMA page_size=4096; PRAGMA journal_mode={journal}; PRAGMA synchronous=FULL; PRAGMA wal_autocheckpoint=0; PRAGMA cache_size=-8192; PRAGMA mmap_size=0; CREATE TABLE IF NOT EXISTS records (key BLOB PRIMARY KEY, value BLOB NOT NULL) WITHOUT ROWID;"))?;
                let actual: String = db.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
                if !actual.eq_ignore_ascii_case(journal) {
                    return Err(format!("requested SQLite {journal}, got {actual}").into());
                }
                for (pragma, expected) in [
                    ("page_size", 4096),
                    ("synchronous", 2),
                    ("wal_autocheckpoint", 0),
                    ("cache_size", -8192),
                    ("mmap_size", 0),
                ] {
                    let actual: i64 =
                        db.query_row(&format!("PRAGMA {pragma}"), [], |row| row.get(0))?;
                    if actual != expected {
                        return Err(format!("unexpected SQLite {pragma}: {actual}").into());
                    }
                }
                Ok(Self::Sqlite(db))
            }
            _ => Err(format!("unknown engine {kind}").into()),
        }
    }

    pub fn apply(&mut self, batch: &Batch, before_commit: impl FnOnce()) -> Result<()> {
        match self {
            Self::Redb(db, _) => {
                let mut tx = db.begin_write()?;
                tx.set_durability(redb::Durability::Immediate)?;
                {
                    let mut table = tx.open_table(TABLE)?;
                    for (key, value) in batch {
                        table.insert(key.as_slice(), value.as_slice())?;
                    }
                }
                before_commit();
                tx.commit()?;
            }
            Self::Sqlite(db) => {
                let tx = db.transaction()?;
                {
                    let mut insert = tx.prepare("INSERT INTO records(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value")?;
                    for (key, value) in batch {
                        insert.execute(params![key, value])?;
                    }
                }
                before_commit();
                tx.commit()?;
            }
        }
        Ok(())
    }

    pub fn scan(&self, start: &[u8], end: &[u8]) -> Result<Records> {
        match self {
            Self::Redb(db, _) => {
                let tx = db.begin_read()?;
                let table = tx.open_table(TABLE)?;
                table
                    .range(start..end)?
                    .map(|entry| {
                        let (key, value) = entry?;
                        Ok((key.value().to_vec(), value.value().to_vec()))
                    })
                    .collect()
            }
            Self::Sqlite(db) => {
                let mut statement = db.prepare(
                    "SELECT key,value FROM records WHERE key>=?1 AND key<?2 ORDER BY key",
                )?;
                let rows = statement
                    .query_map(params![start, end], |row| Ok((row.get(0)?, row.get(1)?)))?;
                rows.map(|row| Ok(row?)).collect()
            }
        }
    }

    pub fn reset_counts(&self) {
        if let Self::Redb(_, backend) = self {
            backend.reset();
        }
    }
    pub fn counts(&self) -> Option<Counts> {
        match self {
            Self::Redb(_, backend) => Some(backend.counts()),
            Self::Sqlite(_) => None,
        }
    }
    pub fn deny_growth(&self, path: &Path) -> Result<()> {
        match self {
            Self::Redb(_, backend) => backend.limit(std::fs::metadata(path)?.len()),
            Self::Sqlite(db) => {
                let pages: i64 = db.query_row("PRAGMA page_count", [], |row| row.get(0))?;
                db.execute_batch(&format!("PRAGMA max_page_count={pages}"))?;
            }
        }
        Ok(())
    }

    pub fn checkpoint(&self) -> Result<()> {
        if let Self::Sqlite(db) = self {
            let busy: i64 =
                db.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
            if busy != 0 {
                return Err("SQLite checkpoint was blocked".into());
            }
        }
        Ok(())
    }
}

pub fn file_bytes(path: &Path) -> Result<(u64, u64)> {
    let main = std::fs::metadata(path)?.len();
    let mut sidecars = 0;
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut name = path.as_os_str().to_os_string();
        name.push(suffix);
        match std::fs::metadata(name) {
            Ok(m) => sidecars += m.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok((main, sidecars))
}
