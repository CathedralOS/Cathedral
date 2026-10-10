//! Bounded page buffers; no allocator or ambient memory authority.
//! Sealing consumes write access. Accepted readers retain a lease even when the
//! producer exits. Completion must release the reader before the producer frees.
pub mod raw;
use crate::{Error, link::Link};
use cathedral_contracts::memory::PAGE_BYTES;

struct Region {
    handle: u64,
    address: u64,
    bytes: usize,
}
impl Drop for Region {
    fn drop(&mut self) {
        // SAFETY: This unique wrapper owns its mapping and all borrows ended.
        // A pending peer can make owner release BUSY. Such an abandoned owner
        // reference is bounded by its budget and reclaimed at task exit.
        unsafe { raw::release(self.handle) }.ok();
    }
}
pub struct Private(Region);
pub struct Sealed(Region);
pub struct Shared(Region);
impl Private {
    pub fn allocate(pages: usize) -> Result<Self, Error> {
        let handle = raw::allocate(pages)?;
        Ok(Self(Region {
            handle,
            address: raw::address(handle)?,
            bytes: pages * PAGE_BYTES,
        }))
    }
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY: Fresh zeroed, uniquely owned writable pages. The mutable borrow
        // prevents sealing/releasing until every write borrow has ended.
        unsafe { core::slice::from_raw_parts_mut(self.0.address as *mut u8, self.0.bytes) }
    }
    pub fn seal(self, peer: Link) -> Result<Sealed, Error> {
        // SAFETY: Consuming self ends all mutable borrows and transfers ownership.
        unsafe { raw::seal(self.0.handle, peer.raw()) }?;
        Ok(Sealed(self.0))
    }
}
impl Sealed {
    /// Token to send over the authorized peer link, never a remote pointer.
    pub fn handle(&self) -> u64 {
        self.0.handle
    }
    /// Call after consumer completion. BUSY means the peer still holds authority.
    pub fn release(self) -> Result<(), Error> {
        // SAFETY: Consumes the owner wrapper, with no outstanding borrows.
        unsafe { raw::release(self.0.handle) }?;
        core::mem::forget(self);
        Ok(())
    }
}
impl Shared {
    /// Accept only an offer from this service-side link's current producer.
    pub fn accept_from(handle: u64, link: Link) -> Result<Self, Error> {
        let address = raw::accept_from(handle, link.raw())?;
        Ok(Self(Region {
            handle,
            address,
            bytes: raw::pages(handle)? * PAGE_BYTES,
        }))
    }
    pub fn accept(handle: u64) -> Result<Self, Error> {
        let address = raw::accept(handle)?;
        Ok(Self(Region {
            handle,
            address,
            bytes: raw::pages(handle)? * PAGE_BYTES,
        }))
    }
    pub fn bytes(&self) -> &[u8] {
        // SAFETY: Kernel pins immutable, non-executable pages until this mapping
        // is released or this task dies; producer failure cannot revoke the lease.
        unsafe { core::slice::from_raw_parts(self.0.address as *const u8, self.0.bytes) }
    }
}
