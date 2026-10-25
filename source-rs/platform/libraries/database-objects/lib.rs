#![no_std]
#![forbid(unsafe_code)]
//! Private object roots over immutable content. A root holds numbered objects
//! arranged as a tree: every object has a parent, so a subtree is a realm that
//! can be delegated and copied. Chunks and manifests are shared within the
//! root, so a copy writes only heads, events and edges. The caller supplies a
//! trusted root binding, never an application-selected root or a globally
//! meaningful hash.
extern crate alloc;
mod collection;
mod hierarchy;
pub use cathedral_platform_contracts::storage::metadata::Metadata;
mod objects;
mod publication;
mod reading;
mod records;
use cathedral_storage_engine::redb::{self, Database, ReadableDatabase, TableDefinition};
pub use collection::{
    Collection, PIN_SLOTS, WORKSPACE_BYTES, collect_records, collect_records_within,
};
pub use records::{Head, Object};

/// One native program or bulk payload fits the kernel's bounded shared object.
/// Root quota and the caller's actual funding remain independent admission gates.
pub const MAX_BYTES: usize = cathedral_platform_contracts::storage::MAX_BYTES;
/// Bytes per content chunk. A chunk record is this many value bytes, its
/// 34-byte key and 12 bytes of leaf bookkeeping (a page header and two
/// lengths), which fill one 4 KiB engine page; a 4 KiB chunk was a
/// single-large-value leaf of two pages, so a program read moved twice its
/// bytes through the cache and evicted the metadata working set.
pub const CHUNK_BYTES: usize = 4096 - 64;
pub const INLINE_BYTES: usize = 128;
/// Retained record bytes per root, including keys and event history. This is an
/// admission limit, not a claim about physical redb space or allocator overhead.
/// It also bounds one collection pass's scan.
// Installed code and saved nested snapshots share this lab allowance with data.
// The volume's physical extent remains an independent capacity limit.
pub const ROOT_QUOTA: u64 = 4 * 1024 * 1024;
/// Objects one tree copy may produce; it bounds one request's work, not the
/// root's size.
pub const COPY_LIMIT: usize = 64;
pub const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("private-objects-v3");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    TooLarge,
    Conflict,
    Capacity,
    /// A pre-commit extent allocation was refused. The request did not publish,
    /// but redb must be dropped and reopened before another operation.
    ReopenCapacity,
    Invalid,
    Missing,
    /// The object still has children; delete them first.
    NotEmpty,
    /// One window of live identities would exceed the collection workspace
    /// even at the finest split. Nothing was removed; the store is unchanged.
    Workspace,
    Corrupt,
    Engine,
}
#[derive(Clone, Copy)]
pub enum Stage {
    Prepared,
    /// Inside the writer, immediately before commit; observers must not reenter.
    Committing,
    Committed,
}
pub type Result<T> = core::result::Result<T, Error>;
fn engine<T, E: Into<redb::Error>>(result: core::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| {
        if cathedral_storage_engine::capacity_refusal(error.into()) {
            Error::ReopenCapacity
        } else {
            Error::Engine
        }
    })
}
/// Commit failures have an uncertain outcome, even if they mention capacity.
/// Never report a definite refusal after crossing the publication boundary.
fn commit(tx: redb::WriteTransaction) -> Result<()> {
    tx.commit().map_err(|_| Error::Engine)
}

pub struct Store<'a> {
    db: &'a Database,
}
impl<'a> Store<'a> {
    /// Called only after the ready volume and engine identity have been checked.
    pub fn open(db: &'a Database) -> Result<Self> {
        // Reopening a verified store does not need a writer or fresh disk space.
        {
            let read = engine(db.begin_read())?;
            match read.open_table(TABLE) {
                Ok(_) => return Ok(Self { db }),
                Err(redb::TableError::TableDoesNotExist(_)) => (),
                Err(_) => return Err(Error::Engine),
            }
        }
        let mut tx = engine(db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        engine(tx.open_table(TABLE))?;
        commit(tx)?;
        Ok(Self { db })
    }
    fn writer(&self) -> Result<redb::WriteTransaction> {
        let mut tx = engine(self.db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        Ok(tx)
    }
}

#[cfg(all(test, feature = "host"))]
extern crate std;
#[cfg(all(test, feature = "host"))]
mod collection_tests;
#[cfg(all(test, feature = "host"))]
mod tests;

#[cfg(all(test, feature = "host"))]
mod capacity_tests;
