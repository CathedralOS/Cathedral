//! Requested live bytes (not allocator overhead/RSS), measured without allocating.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
pub struct Meter;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static LARGEST: AtomicUsize = AtomicUsize::new(0);
static ACTIVE: AtomicBool = AtomicBool::new(false);
fn add(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Relaxed) + bytes;
    if ACTIVE.load(Relaxed) {
        PEAK.fetch_max(live, Relaxed);
        LARGEST.fetch_max(bytes, Relaxed);
    }
}
// SAFETY: All allocations retain System's pointer/layout ownership; atomics merely
// observe successful allocations and matching releases, without allocating.
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forward the caller's valid layout unchanged.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            add(layout.size());
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        // SAFETY: Pointer and layout belong to this System-backed allocator.
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: Forward the live allocation and requested size unchanged.
        let next = unsafe { System.realloc(ptr, layout, size) };
        if !next.is_null() {
            LIVE.fetch_sub(layout.size(), Relaxed);
            add(size);
        }
        next
    }
}
pub fn live() -> usize {
    LIVE.load(Relaxed)
}
pub fn begin() {
    PEAK.store(live(), Relaxed);
    LARGEST.store(0, Relaxed);
    ACTIVE.store(true, Relaxed);
}
pub fn report(phase: &str, baseline: usize) {
    ACTIVE.store(false, Relaxed);
    println!(
        "{phase},{},{},{}",
        live().saturating_sub(baseline),
        PEAK.load(Relaxed).saturating_sub(baseline),
        LARGEST.load(Relaxed)
    );
}
