//! Explicit process-lifetime heap installation over one or more funded page
//! objects. Page ownership stays with the kernel account until process
//! teardown, including panic/fault/cancellation.
use crate::{Error, abi, memory::raw};
use cathedral_kernel_contracts::memory::{MAX_PAGES, PAGE_BYTES};
pub use cathedral_user_heap::{Heap, MAX_EXTENTS, Stats};

/// Install `pages` of heap from as many private objects as the per-object cap
/// requires, at most `MAX_EXTENTS`. An object whose installation fails is
/// released; extents installed before it stay owned until process death, and
/// callers treat the error as fatal.
pub fn initialize(heap: &'static Heap, pages: usize) -> Result<(), Error> {
    let objects = pages.div_ceil(MAX_PAGES);
    if pages == 0 || objects > MAX_EXTENTS {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    let mut remaining = pages;
    for index in 0..objects {
        let count = remaining.min(MAX_PAGES);
        let handle = raw::allocate(count)?;
        let installed = install(heap, handle, index == 0);
        if installed.is_err() {
            // SAFETY: This object's installation failed, so the heap holds no
            // reference to it and nothing was borrowed from it.
            unsafe { raw::release(handle) }.ok();
            return installed;
        }
        remaining -= count;
    }
    Ok(())
}
fn install(heap: &'static Heap, handle: u64, first: bool) -> Result<(), Error> {
    let address = raw::address(handle)? as *mut u8;
    let bytes = raw::pages(handle)? * PAGE_BYTES;
    // SAFETY: Fresh uniquely owned RW/NX mapping, disjoint from every other
    // object. This module never exposes its handles or releases success; the
    // kernel reclaims them on process death.
    let installed = unsafe {
        if first {
            heap.initialize(address, bytes)
        } else {
            heap.extend(address, bytes)
        }
    };
    installed.map_err(|_| Error(abi::INVALID_ARGUMENT as i64))
}
