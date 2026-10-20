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
use cathedral_storage_engine::redb::{self, Database, TableDefinition};
pub use collection::{Collection, PIN_SLOTS, collect_records};
pub use records::{Head, Object};

/// Largest object: 64 pages, so a program or a bulk reply fits in half a
/// task's page allowance while the other half holds what it already retains.
pub const MAX_BYTES: usize = 256 * 1024;
pub const CHUNK_BYTES: usize = 4096;
pub const INLINE_BYTES: usize = 128;
/// Retained record bytes per root, including keys and event history. This is an
/// admission limit, not a claim about physical redb space or allocator overhead.
/// It also bounds one collection pass's scan.
// Installed code and saved nested snapshots share this lab allowance with data.
// The volume's physical extent remains an independent capacity limit.
pub const ROOT_QUOTA: u64 = 2 * 1024 * 1024;
/// Objects one tree copy may produce; it bounds one request's work, not the
/// root's size.
pub const COPY_LIMIT: usize = 64;
pub const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("private-objects-v3");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    TooLarge,
    Conflict,
    Capacity,
    Invalid,
    Missing,
    /// The object still has children; delete them first.
    NotEmpty,
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
fn engine<T, E>(result: core::result::Result<T, E>) -> Result<T> {
    result.map_err(|_| Error::Engine)
}

pub struct Store<'a> {
    db: &'a Database,
}
impl<'a> Store<'a> {
    /// Called only after the ready volume and engine identity have been checked.
    pub fn open(db: &'a Database) -> Result<Self> {
        let mut tx = engine(db.begin_write())?;
        engine(tx.set_durability(redb::Durability::Immediate))?;
        engine(tx.open_table(TABLE))?;
        engine(tx.commit())?;
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
