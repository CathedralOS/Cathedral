#![no_std]
#![no_main]
//! Private object storage provider. Only this executable receives the disk
//! grant. Fund the heap, recover the engine, serve two private roots.
mod bulk;
#[cfg(feature = "recovery-lab")]
mod gate;
mod metrics;
mod service;
use cathedral_user_runtime::heap::{self, Heap};
#[global_allocator]
static HEAP: Heap = Heap::empty();
cathedral_user_runtime::entry!(main);
fn main(_: u64, _: u64) -> u64 {
    // Leave the rest of the private allowance for sealed read replies of up to
    // MAX_BYTES, separately from the allocator. Reader custody continues to
    // charge this provider's budget until the last reader releases a reply.
    heap::initialize(&HEAP, 56).unwrap();
    let disk = cathedral_ata_pio::Ata::open().unwrap();
    let db = cathedral_storage_engine::open(disk, 4096, 4 * 1024 * 1024, 32768).unwrap();
    metrics::report("recovered");
    {
        let store = cathedral_database_objects::Store::open(&db).unwrap();
        service::serve(&store);
    }
    assert_eq!(
        cathedral_user_runtime::resources::budget()
            .unwrap()
            .used
            .reader_pages,
        0
    );
    drop(db);
    metrics::report("closed");
    assert_eq!(HEAP.stats().live, 0);
    0
}
