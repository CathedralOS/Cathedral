//! Experimental page objects with caller-local names and guarded local mapping slots.
//! ALLOCATE(pages) returns a handle to zeroed private RW/NX pages; ADDRESS(handle)
//! and PAGES(handle) query only caller-held authority. ADDRESS requires a mapping.
//! RELEASE removes that local mapping and right. Unknown/foreign names return BAD_HANDLE.
//!
//! FREEZE(handle) irreversibly seals without a boot link. IPC_SEND_MEMORY copies a
//! read capability over an ordinary endpoint; IPC_RECEIVE_MEMORY installs a fresh
//! local handle without a mapping. Queued rights reserve recipient reader capacity.
//! MAP_OBJECT maps RO/NX; UNMAP_OBJECT drops only that mapping. All holders may
//! forward; surviving rights keep backing and its original sponsor charge alive.
//! The final release/death/discard refunds backing. No live reader is remotely unmapped.
//!
//! Legacy boot-link compatibility path:
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
/// Maximum pages in one object.
pub const MAX_PAGES: usize = 128;
/// A subtree owner may fund several objects and descendant accounts.
pub const MAX_PRIVATE_PAGES: usize = 8 * MAX_PAGES;
/// Maximum per-task reader page budget. Reader pages pin other tasks' backing
/// rather than funding frames, and a matrix hub needs far more of them than
/// any one object: it retains every image its subtree shows, holds old and new
/// during each replacement, and lends the same kind of budget to each nested
/// matrix below it.
pub const MAX_READER_PAGES: usize = 12 * MAX_PAGES;
/// Separate metadata limits: larger page grants must not buy every global slot.
/// A matrix or the compositor retains one sealed image per surface it shows
/// and holds its replacement beside it, including images below nested matrices.
pub const MAX_PRIVATE_REGIONS: usize = 4;
pub const MAX_SHARED_REGIONS: usize = 48;
pub const MAX_MAPPING_SLOTS: usize = MAX_PRIVATE_REGIONS + MAX_SHARED_REGIONS;
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
    /// Reader pages, including queued/unmapped rights and rights retained after producer death.
    pub shared_pages: usize,
}

/// Includes queued/unmapped read capabilities, bounded independently of backing.
/// Retirement tracks them in a 128-bit mask.
/// The global pool is independent of each task's local slot allowance.
pub const MAX_READ_HANDLES: usize = 128;
const _: () = assert!(MAX_READ_HANDLES <= 128);
