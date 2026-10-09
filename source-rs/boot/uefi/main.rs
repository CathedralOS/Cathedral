#![no_std]
#![no_main]

use cathedral_contracts::boot::{BootMemory, MemoryKind, MemoryRegion};
use cathedral_core::extent::FrameAllocator;
use cathedral_uart_16550::SerialPort;
use cathedral_x86_64::{disable_interrupts, halt_forever};
use core::{fmt::Write, panic::PanicInfo};
use uefi::{
    Status, boot, entry,
    mem::memory_map::{MemoryAttribute, MemoryDescriptor, MemoryMap, MemoryType},
};

#[entry]
fn main() -> Status {
    // SAFETY: The runner selects a single-CPU QEMU PC with COM1. This boot
    // application is privileged; it is the only Rust owner of the UART.
    let mut serial = unsafe { SerialPort::com1() };
    writeln!(serial, "Cathedral Rust lab: UEFI entry").ok();

    // No UEFI protocols, pool-owning values, logger, or global allocator are
    // retained here. The crate captures the final map and retries a stale key.
    // SAFETY: All boot-services resources except the returned map are absent.
    let firmware_map = unsafe { boot::exit_boot_services(None) };
    // SAFETY: Firmware has handed control to this single boot CPU. Keep IRQs
    // masked until Cathedral installs its own exception and interrupt tables.
    unsafe {
        disable_interrupts();
    }
    writeln!(serial, "Cathedral Rust lab: ExitBootServices complete").ok();

    assert_eq!(firmware_map.meta().desc_version, MemoryDescriptor::VERSION);
    let mut memory = BootMemory::new();
    for descriptor in firmware_map.entries() {
        // UEFI memory attribute bits: RUNTIME, HOT_PLUGGABLE, and SP. Match the
        // Omega bootstrap's conservative ordinary-RAM policy. All loader,
        // firmware, ACPI, MMIO and runtime regions remain reserved, including
        // our image, live firmware stack/page tables and the final map buffer.
        let ineligible = MemoryAttribute::RUNTIME
            | MemoryAttribute::HOT_PLUGGABLE
            | MemoryAttribute::SPECIAL_PURPOSE;
        let usable =
            descriptor.ty == MemoryType::CONVENTIONAL && !descriptor.att.intersects(ineligible);
        memory
            .push(MemoryRegion {
                base: descriptor.phys_start,
                page_count: descriptor.page_count,
                kind: if usable {
                    MemoryKind::Usable
                } else {
                    MemoryKind::Reserved
                },
            })
            .expect("invalid or oversized firmware memory inventory");
    }
    writeln!(
        serial,
        "Cathedral Rust lab: memory regions={} usable_bytes={}",
        memory.regions().len(),
        memory.usable_bytes()
    )
    .ok();
    let mut frames = FrameAllocator::new(memory);
    let frame = frames.allocate().expect("no usable physical frame");
    writeln!(
        serial,
        "Cathedral Rust lab: first frame={:#x}",
        frame.address()
    )
    .ok();
    // This milestone allocates a frame in accounting only. Mapping and actual
    // access await Cathedral-owned page tables; no address-to-reference cast.
    writeln!(serial, "CATHEDRAL_RS_BOOT_OK").ok();

    #[cfg(feature = "smoke-test")]
    // SAFETY: Only the smoke runner supplies the debug-exit device at 0xf4.
    unsafe {
        cathedral_x86_64::qemu_exit(0x10)
    }

    #[cfg(not(feature = "smoke-test"))]
    // SAFETY: No firmware return is valid after exit. This CPU enters idle.
    unsafe {
        halt_forever()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    // SAFETY: Fatal path on the sole CPU. Abandon the interrupted UART owner
    // permanently; no unwinding or resumption can race this emergency writer.
    unsafe {
        disable_interrupts();
        let mut serial = SerialPort::com1();
        writeln!(serial, "CATHEDRAL_RS_PANIC: {info}").ok();
        halt_forever()
    }
}
