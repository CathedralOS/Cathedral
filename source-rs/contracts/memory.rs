//! Experimental page objects. Fixed virtual slots have an unmapped trailing guard.
//! Handles identify an allocation incarnation; raw addresses carry no authority.
//! ALLOCATE(pages) returns a handle to zeroed private RW/NX pages; ADDRESS(handle)
//! and PAGES(handle) query a caller-owned or accepted mapping. RELEASE(handle)
//! removes only the caller's mapping. Unmapped and stale handles fail closed.
//!
//! SEAL(handle, link) irreversibly removes producer WRITE and offers the object
//! to that live boot-approved service. MAP(handle) accepts a single RO/NX lease
//! and returns its address. Producer release is BUSY until peer completion/death.
//! An accepted reader survives producer death; an unaccepted offer does not.
//! MAP(handle, service_link) additionally checks the offer's producer against
//! that service-side link before accepting. Zero retains the original MAP behavior.
//! Reader release/death plus owner release/death frees backing. No live reader
//! is remotely unmapped. These are lab semantics, not Cathedral's stable ABI.
pub const PAGE_BYTES: usize = 4096;
pub const MAX_PAGES: usize = 4;
pub const MAX_REGIONS: usize = 32;
pub const USER_BASE: u64 = 0x0000_0080_0040_0000;
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
