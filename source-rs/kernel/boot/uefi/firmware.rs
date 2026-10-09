//! The UEFI crate owns the firmware ABI and exit transaction. This adapter
//! applies Cathedral's conservative memory policy to its returned inventory.

use cathedral_contracts::boot::{BootMemory, MemoryKind, MemoryRegion};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;
use uefi::mem::memory_map::{MemoryAttribute, MemoryDescriptor, MemoryMap, MemoryType};

pub struct BootInfo {
    pub framebuffer: Option<cathedral_contracts::display::Framebuffer>,
    pub inventory: BootMemory,
    pub image: cathedral_arch::ImageRange,
}

pub fn exit_boot_services(console: &mut SerialPort) -> BootInfo {
    let framebuffer = crate::graphics::capture();
    let image = {
        let loaded = uefi::boot::open_protocol_exclusive::<uefi::proto::loaded_image::LoadedImage>(
            uefi::boot::image_handle(),
        )
        .expect("loaded image protocol unavailable");
        let (base, bytes) = loaded.info();
        cathedral_arch::ImageRange {
            base: base as u64,
            bytes,
        }
    };
    // SAFETY: No protocol references, pool-owning values, firmware logger or
    // global allocator survive this point. The crate owns map-key retry logic.
    let firmware_map = unsafe { uefi::boot::exit_boot_services(None) };
    // SAFETY: Firmware relinquished this sole boot CPU; no kernel IDT exists yet.
    unsafe {
        cathedral_arch::disable_interrupts();
    }
    writeln!(console, "Cathedral Rust lab: ExitBootServices complete").ok();

    assert_eq!(firmware_map.meta().desc_version, MemoryDescriptor::VERSION);
    let mut inventory = BootMemory::new();
    for descriptor in firmware_map.entries() {
        let ineligible = MemoryAttribute::RUNTIME
            | MemoryAttribute::HOT_PLUGGABLE
            | MemoryAttribute::SPECIAL_PURPOSE;
        // Keep the entire descriptor reserved if firmware advertises framebuffer
        // storage as conventional RAM. Device pages must never enter our allocator.
        let display_overlap = framebuffer.is_some_and(|fb| {
            descriptor.phys_start < fb.physical + fb.bytes
                && fb.physical
                    < descriptor
                        .phys_start
                        .saturating_add(descriptor.page_count.saturating_mul(4096))
        });
        let usable = !display_overlap
            && descriptor.ty == MemoryType::CONVENTIONAL
            && !descriptor.att.intersects(ineligible);
        // All loader/firmware storage remains reserved, retaining the image,
        // current stack, inherited tables and map buffer through the handoff.
        inventory
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
        console,
        "Cathedral Rust lab: memory regions={} usable_bytes={}",
        inventory.regions().len(),
        inventory.usable_bytes()
    )
    .ok();
    BootInfo {
        inventory,
        image,
        framebuffer,
    }
}
