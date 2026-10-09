//! Reclaiming heap using the upstream allocator. IRQ masking encloses lock
//! ownership so a preempted task cannot strand the single CPU inside allocation.
//! Fatal/NMI handlers and the scheduler never allocate.

use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicBool, Ordering},
};
use linked_list_allocator::LockedHeap;

pub struct KernelHeap {
    heap: LockedHeap,
    initialized: AtomicBool,
}

impl Default for KernelHeap {
    fn default() -> Self {
        Self::empty()
    }
}

impl KernelHeap {
    pub const fn empty() -> Self {
        Self {
            heap: LockedHeap::empty(),
            initialized: AtomicBool::new(false),
        }
    }

    /// # Safety
    /// Called once before task scheduling. The range is exclusively owned,
    /// writable, permanently mapped RAM, large enough for allocator metadata.
    /// All use of this allocator requires kernel privilege.
    pub unsafe fn initialize(&self, base: *mut u8, bytes: usize) {
        assert!(
            !self.initialized.swap(true, Ordering::AcqRel),
            "heap already initialized"
        );
        // SAFETY: Caller supplies exclusive permanent backing; masking prevents
        // task switches while the allocator's internal lock is held.
        unsafe {
            cathedral_arch::without_interrupts(|| self.heap.lock().init(base, bytes));
        }
    }

    /// # Safety
    /// Requires kernel privilege and execution outside NMI/fatal handlers.
    pub unsafe fn used(&self) -> usize {
        // SAFETY: The caller provides privilege; the closure cannot suspend.
        unsafe { cathedral_arch::without_interrupts(|| self.heap.lock().used()) }
    }
}

// SAFETY: Upstream enforces layout/reclamation. Exclusive initialized backing
// and privileged use are initialization obligations; all lock access masks IRQs.
unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forward GlobalAlloc's layout contract under the kernel IRQ guard.
        unsafe { cathedral_arch::without_interrupts(|| self.heap.alloc(layout)) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: Caller supplies the live allocation and its original layout.
        unsafe {
            cathedral_arch::without_interrupts(|| self.heap.dealloc(pointer, layout));
        }
    }
}
