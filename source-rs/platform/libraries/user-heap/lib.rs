#![no_std]
//! Opt-in heap over explicitly supplied, permanently owned memory grants. The
//! kernel funds pages; this allocator only subdivides those bytes. No syscalls
//! or hidden growth occur under its lock. Runtime integration lives in
//! user-runtime/heap.rs.
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use spinning_top::Spinlock;

/// Extents one heap may span. Each is one funded page object, so this bounds
/// the mapping slots a heap can take from its process.
pub const MAX_EXTENTS: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub capacity: usize,
    pub live: usize,
    pub peak: usize,
    pub largest: usize,
    pub failures: usize,
}
struct State {
    extents: [linked_list_allocator::Heap; MAX_EXTENTS],
    installed: usize,
    stats: Stats,
}
pub struct Heap(Spinlock<State>);
impl Heap {
    pub const fn empty() -> Self {
        Self(Spinlock::new(State {
            extents: [const { linked_list_allocator::Heap::empty() }; MAX_EXTENTS],
            installed: 0,
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
        if state.installed != 0 {
            return Err("heap already initialized");
        }
        // SAFETY: The caller's obligations are this function's.
        unsafe { state.install(base, bytes) }
    }
    /// Add another extent after `initialize`. Allocation prefers earlier
    /// extents and a free returns to the extent that holds it.
    /// # Safety
    /// As for `initialize`; the bytes must not overlap an installed extent.
    pub unsafe fn extend(&self, base: *mut u8, bytes: usize) -> Result<(), &'static str> {
        let mut state = self.0.lock();
        if state.installed == 0 {
            return Err("heap not initialized");
        }
        // SAFETY: The caller's obligations are this function's.
        unsafe { state.install(base, bytes) }
    }
    pub fn stats(&self) -> Stats {
        self.0.lock().stats
    }
}
impl State {
    /// # Safety
    /// As for `Heap::initialize`.
    unsafe fn install(&mut self, base: *mut u8, bytes: usize) -> Result<(), &'static str> {
        if self.installed == MAX_EXTENTS {
            return Err("heap extents exhausted");
        }
        if base.is_null()
            || !(base as usize).is_multiple_of(core::mem::align_of::<usize>())
            || bytes < 2 * core::mem::size_of::<usize>()
            || (base as usize).checked_add(bytes).is_none()
        {
            return Err("invalid heap extent");
        }
        let (start, end) = (base as usize, base as usize + bytes);
        if self.extents[..self.installed]
            .iter()
            .any(|extent| start < extent.top() as usize && (extent.bottom() as usize) < end)
        {
            return Err("overlapping heap extent");
        }
        // SAFETY: Caller owns and pins the checked range, which overlaps no
        // installed extent; this slot has never been initialized.
        unsafe {
            self.extents[self.installed].init(base, bytes);
        }
        self.stats.capacity += self.extents[self.installed].size();
        self.installed += 1;
        Ok(())
    }
    fn holder(&mut self, pointer: *mut u8) -> Option<&mut linked_list_allocator::Heap> {
        let installed = self.installed;
        self.extents[..installed]
            .iter_mut()
            .find(|extent| extent.bottom() <= pointer && pointer < extent.top())
    }
}
// SAFETY: The lock serializes all free-list access. Only the exclusively supplied
// extents are managed, and deallocation requires the original live pointer/layout.
unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut state = self.0.lock();
        let installed = state.installed;
        let allocation = state.extents[..installed]
            .iter_mut()
            .find_map(|extent| extent.allocate_first_fit(layout).ok());
        if let Some(pointer) = allocation {
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
        let extent = state
            .holder(ptr)
            .expect("freed pointer outside every heap extent");
        // SAFETY: GlobalAlloc caller returns the same live allocation and layout,
        // to the extent that produced it.
        unsafe {
            extent.deallocate(NonNull::new_unchecked(ptr), layout);
        }
        state.stats.live -= layout.size();
    }
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
