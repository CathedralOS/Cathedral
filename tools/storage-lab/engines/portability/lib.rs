//! Compile-only check of redb's experimental no_std backend entrance. No runtime
//! allocator, locking, device adapter or Cathedral guest integration is supplied.
#![no_std]

pub fn open(backend: impl redb::StorageBackend) -> Result<redb::Database, redb::DatabaseError> {
    redb::Database::builder().create_with_backend(backend)
}
