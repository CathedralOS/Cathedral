#![no_std]
#![forbid(unsafe_code)]
//! Bounded private-object experiment: publish immutable content, revision and
//! change together; read verifies that chain. The caller supplies a trusted root
//! binding, never an application-selected root or globally meaningful hash.
extern crate alloc;
mod publication;
mod reading;
mod records;
use cathedral_storage_engine::redb::{self, Database, TableDefinition};
pub use records::Head;

pub const MAX_BYTES: usize = 16 * 1024;
pub const CHUNK_BYTES: usize = 4096;
pub const INLINE_BYTES: usize = 128;
/// Retained record bytes per root, including keys and event history. This is an
/// admission limit, not a claim about physical redb space or allocator overhead.
pub const ROOT_QUOTA: u64 = 24 * 1024;
pub const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("private-objects-v1");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    TooLarge,
    Conflict,
    Capacity,
    Invalid,
    Corrupt,
    Engine,
}
#[derive(Clone, Copy)]
pub enum Stage {
    Prepared,
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
}

#[cfg(all(test, feature = "host"))]
extern crate std;
#[cfg(all(test, feature = "host"))]
mod tests;
