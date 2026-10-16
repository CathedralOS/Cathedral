#![no_std]
//! Opt-in heap over one explicitly supplied, permanently owned memory grant.
//! The kernel funds pages; this allocator only subdivides those bytes. No syscalls
//! or hidden growth occur under its lock. Runtime integration lives in user-runtime/heap.rs.
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use spinning_top::Spinlock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub capacity: usize,
    pub live: usize,
    pub peak: usize,
    pub largest: usize,
    pub failures: usize,
}
struct State {
    arena: linked_list_allocator::Heap,
    stats: Stats,
}
pub struct Heap(Spinlock<State>);
impl Heap {
    pub const fn empty() -> Self {
        Self(Spinlock::new(State {
            arena: linked_list_allocator::Heap::empty(),
            stats: Stats {
                capacity: 0,
                live: 0,
                peak: 0,
                largest: 0,
                failures: 0,
            },
        }))
    }
    /// # Safety
    /// Supply exclusive, aligned writable bytes, pinned for this heap's entire
    /// lifetime. No external access/release may overlap any allocator use.
    pub unsafe fn initialize(&self, base: *mut u8, bytes: usize) -> Result<(), &'static str> {
        let mut state = self.0.lock();
        if state.stats.capacity != 0 {
            return Err("heap already initialized");
        }
        if base.is_null()
            || !(base as usize).is_multiple_of(core::mem::align_of::<usize>())
            || bytes < 2 * core::mem::size_of::<usize>()
            || (base as usize).checked_add(bytes).is_none()
        {
            return Err("invalid heap extent");
        }
        // SAFETY: Caller owns and pins the checked range; no old heap is installed.
        unsafe {
            state.arena.init(base, bytes);
        }
        state.stats.capacity = state.arena.size();
        Ok(())
    }
    pub fn stats(&self) -> Stats {
        self.0.lock().stats
    }
}
// SAFETY: The lock serializes all free-list access. Only the exclusively supplied
// extent is managed, and deallocation requires the original live pointer/layout.
unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut state = self.0.lock();
        let allocation = state.arena.allocate_first_fit(layout);
        if let Ok(pointer) = allocation {
            state.stats.live += layout.size();
            state.stats.peak = state.stats.peak.max(state.stats.live);
            state.stats.largest = state.stats.largest.max(layout.size());
            pointer.as_ptr()
        } else {
            state.stats.failures += 1;
            core::ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut state = self.0.lock();
        // SAFETY: GlobalAlloc caller returns the same live allocation and layout.
        unsafe {
            state.arena.deallocate(NonNull::new_unchecked(ptr), layout);
        }
        state.stats.live -= layout.size();
    }
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
