//! Reopen the existing granted volume; never format on a runtime failure.
pub fn open() -> cathedral_storage_engine::redb::Database {
    let disk = cathedral_ata_pio::Ata::open().unwrap();
    let db = cathedral_storage_engine::open_observed(
        disk,
        4096,
        8 * 1024 * 1024,
        // Cache competes with writer state and the root's transient live-record
        // set. Leave collection workspace inside the fixed 56-page heap.
        32 * 1024,
        crate::metrics::extent,
    )
    .unwrap();
    cathedral_database_objects::Store::open(&db).unwrap();
    crate::metrics::report("recovered");
    db
}
