//! Experimental page objects with caller-local names and guarded local mapping slots.
//! ALLOCATE(pages) returns a handle to zeroed private RW/NX pages; ADDRESS(handle)
//! and PAGES(handle) query only the caller's installed mapping. RELEASE(handle)
//! removes only that mapping. Unknown and foreign names both return BAD_HANDLE.
//!
//! SEAL(handle, link) irreversibly removes producer WRITE and offers the object
//! to that live boot-approved service. Send the producer-local handle as an offer
//! token over that connection. MAP(offer, service_link) authenticates its producer
//! through the receiving link, accepts a single RO/NX lease and returns a NEW
//! caller-local reader handle. ADDRESS(reader) gives its local virtual address.
//! A link is mandatory: the same numeric offer can exist at different producers.
//!
//! Producer release is BUSY until peer completion/death. Accepted readers survive
//! producer death; unaccepted offers do not. Reader release/death plus owner
//! release/death frees backing. No live reader is remotely unmapped. Names are
//! non-reused within a session (including task-slot reuse); raw addresses may be
//! reused. These are lab semantics, not Cathedral's stable ABI.
pub const PAGE_BYTES: usize = 4096;
/// Maximum pages per object and per-task private/accepted-reader page budget.
pub const MAX_PAGES: usize = 16;
/// Separate metadata limits: larger page grants must not buy every global slot.
pub const MAX_PRIVATE_REGIONS: usize = 4;
pub const MAX_SHARED_REGIONS: usize = 4;
// Every live object has an owner or an accepted reader. This capacity covers
// every task's object limits, including readers whose producer has exited.
pub const MAX_REGIONS: usize = crate::user::MAX_TASKS * (MAX_PRIVATE_REGIONS + MAX_SHARED_REGIONS);
pub const USER_BASE: u64 = 0x0000_0080_0040_0000;
/// A process-local mapping slot, never the global backing arena index.
pub const fn address(index: usize) -> u64 {
    USER_BASE + (index * (MAX_PAGES + 1) * PAGE_BYTES) as u64
}
#[derive(Clone, Copy, Debug)]
pub struct Grant {
    pub task: usize,
    pub private_pages: usize,
    /// Accepted reader pages, including leases retained after producer death.
    pub shared_pages: usize,
}
