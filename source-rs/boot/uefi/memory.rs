//! Boot-time composition of core frame policy and architecture-owned mappings.

use cathedral_arch as arch;
use cathedral_contracts::boot::BootMemory;
use cathedral_core::extent::FrameAllocator;
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

pub struct PreparedMemory {
    pub layout: arch::BootLayout,
    frames: FrameAllocator,
}

pub fn prepare(inventory: BootMemory, console: &mut SerialPort) -> PreparedMemory {
    let mut frames = FrameAllocator::new(inventory);
    // SAFETY: Firmware has exited, IRQs are off, and the sole CPU receives
    // unique conventional frames disjoint from live firmware allocations.
    let layout =
        unsafe { arch::prepare_memory(&mut || frames.allocate().map(|frame| frame.address())) }
            .expect("cannot prepare kernel memory");
    writeln!(
        console,
        "Cathedral Rust lab: arch={} page_tables={} root={:#x}",
        arch::CPU_NAME,
        layout.table_count,
        layout.root_address()
    )
    .ok();
    PreparedMemory { frames, layout }
}

pub fn confirm_handoff(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let stack = arch::stack_pointer();
    assert!((memory.layout.stack_bottom..memory.layout.stack_top).contains(&stack));
    // Keep allocation policy usable across the stack and address-space switch.
    assert!(memory.frames.allocate().is_some());
    writeln!(
        console,
        "Cathedral Rust lab: owned page tables and stack rsp={stack:#x}"
    )
    .ok();
}
