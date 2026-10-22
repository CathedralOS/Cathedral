//! Reopen the existing granted volume; never format on a runtime failure.
/// Engine state, the writer and the bounded collection workspace share the
/// heap with the page cache. They are reserved first; the smallest heap keeps
/// the 32 KiB cache it has always had, and every funded page beyond the
/// reserve becomes cache so repeated program reads stop going to the disk.
const ENGINE_RESERVE_BYTES: usize = 256 * 1024;
const MIN_CACHE_BYTES: usize = 32 * 1024;
fn cache_bytes(heap: usize) -> usize {
    heap.saturating_sub(ENGINE_RESERVE_BYTES)
        .max(MIN_CACHE_BYTES)
}
pub fn open() -> cathedral_storage_engine::redb::Database {
    let disk = cathedral_ata_pio::Ata::open().unwrap();
    let db = cathedral_storage_engine::open_observed(
        disk,
        4096,
        8 * 1024 * 1024,
        cache_bytes(crate::HEAP.stats().capacity),
        crate::metrics::extent,
    )
    .unwrap();
    cathedral_database_objects::Store::open(&db).unwrap();
    crate::metrics::report("recovered");
    db
}
