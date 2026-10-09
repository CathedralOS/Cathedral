//! Install the core allocator over the architecture's owned heap mapping.

use cathedral_arch::BootLayout;
use cathedral_core::heap::KernelHeap;
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

#[global_allocator]
pub static HEAP: KernelHeap = KernelHeap::empty();

pub fn initialize(layout: &BootLayout, console: &mut SerialPort) {
    // SAFETY: The active layout owns this permanently mapped writable RAM;
    // boot calls once, before scheduling or any allocation from Rust's heap.
    unsafe {
        HEAP.initialize(layout.heap_start as *mut u8, layout.heap_bytes);
    }
    writeln!(
        console,
        "Cathedral Rust lab: heap initialized bytes={}",
        layout.heap_bytes
    )
    .ok();
}

#[cfg(feature = "smoke-test")]
pub fn verify(layout: &BootLayout, console: &mut SerialPort) {
    use alloc::{boxed::Box, vec::Vec};
    use core::alloc::{GlobalAlloc, Layout};
    // SAFETY: Kernel context; no interrupts/fault handlers allocate here.
    let baseline = unsafe { HEAP.used() };
    for iteration in 0..8u64 {
        let values: Vec<u64> = (0..2048).map(|index| index ^ iteration).collect();
        assert_eq!(values[1023], 1023 ^ iteration);
        assert_eq!(*Box::new(iteration), iteration);
    }
    let aligned = Layout::from_size_align(1024, 4096).unwrap();
    // SAFETY: Exercise the installed allocator, then free with the same layout.
    unsafe {
        let pointer = HEAP.alloc(aligned);
        assert!(!pointer.is_null());
        assert!((pointer as usize).is_multiple_of(4096));
        pointer.write_bytes(0xa5, 1024);
        assert_eq!(pointer.add(1023).read(), 0xa5);
        HEAP.dealloc(pointer, aligned);
        let oversized = Layout::from_size_align(layout.heap_bytes * 2, 16).unwrap();
        assert!(HEAP.alloc(oversized).is_null());
        assert_eq!(HEAP.used(), baseline);
    }
    writeln!(
        console,
        "Cathedral Rust lab: heap alignment exhaustion and reclamation passed"
    )
    .ok();
}
