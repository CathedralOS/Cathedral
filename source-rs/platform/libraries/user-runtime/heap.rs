//! Explicit process-lifetime heap installation. Page ownership stays with the
//! kernel account until process teardown, including panic/fault/cancellation.
use crate::{Error, abi, memory::raw};
pub use cathedral_user_heap::{Heap, Stats};

pub fn initialize(heap: &'static Heap, pages: usize) -> Result<(), Error> {
    let handle = raw::allocate(pages)?;
    let result = (|| {
        let address = raw::address(handle)?;
        let bytes = raw::pages(handle)? * cathedral_kernel_contracts::memory::PAGE_BYTES;
        // SAFETY: Fresh uniquely owned RW/NX mapping. This module never exposes
        // its handle or releases success; the kernel reclaims it on process death.
        unsafe { heap.initialize(address as *mut u8, bytes) }
            .map_err(|_| Error(abi::INVALID_ARGUMENT as i64))
    })();
    if result.is_err() {
        // SAFETY: Failed installation exposed no allocations or borrows.
        unsafe { raw::release(handle) }.ok();
    }
    result
}
