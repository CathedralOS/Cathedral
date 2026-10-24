#![no_std]
#![no_main]
//! Private object storage provider. Only this executable receives the disk
//! grant. Fund the heap, recover the engine, serve two private roots.
mod bulk;
mod database;
#[cfg(feature = "recovery-lab")]
mod gate;
mod hierarchy;
mod listing;
mod metrics;
mod service;
#[cfg(feature = "startup-probes")]
mod startup_probe;
use cathedral_user_runtime::heap::{self, Heap};
#[global_allocator]
static HEAP: Heap = Heap::empty();
cathedral_user_runtime::entry!(main);
/// Stack and data headroom beside the reply pages.
const HEADROOM_PAGES: usize = 8;
/// The smallest heap the engine, writer and collection workspace have run in.
const MIN_HEAP_PAGES: usize = 56;
/// Private objects the heap may span, of the four this task may own: one
/// stays free for the pages behind a bulk reply while it is being published.
const HEAP_OBJECTS: usize = 2;
const _: () = assert!(HEAP_OBJECTS < cathedral_kernel_contracts::memory::MAX_PRIVATE_REGIONS);
/// Split funding between the engine/cache and outstanding replies. Reserving
/// the protocol's largest reply unconditionally starves a smaller funded cache.
/// Replies still check their actual charge; larger payloads need larger grants.
fn heap_pages(private_pages: u64) -> usize {
    (private_pages as usize)
        .saturating_sub(HEADROOM_PAGES)
        .div_ceil(2)
        .clamp(
            MIN_HEAP_PAGES,
            HEAP_OBJECTS * cathedral_kernel_contracts::memory::MAX_PAGES,
        )
}
fn main(_: u64, _argument: u64) -> u64 {
    #[cfg(feature = "startup-probes")]
    startup_probe::run(_argument);
    let funded = cathedral_user_runtime::resources::budget()
        .unwrap()
        .limits
        .private_pages;
    heap::initialize(&HEAP, heap_pages(funded)).unwrap();
    let db = database::open();
    let db = service::serve(db, database::open);
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
