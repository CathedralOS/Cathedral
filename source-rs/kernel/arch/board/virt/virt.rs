// Fixed QEMU virt profile: no PC transports. CPU paging consumes these physical windows.
use super::Devices;
pub(super) fn devices() -> Devices {
    Devices {
        keyboard: None,
        disk: None,
    }
}
pub(crate) const RAM_START: u64 = 0x4000_0000;
pub(crate) const RAM_END: u64 = 0x8000_0000;
pub(super) const UART: usize = 0x0900_0000;
pub(super) const GIC_DISTRIBUTOR: usize = 0x0800_0000;
pub(super) const GIC_CPU: usize = 0x0801_0000;
pub(crate) const TIMER_IRQ: u32 = 27;

/// Physical windows selected by this board; descriptor encoding belongs to the ISA.
pub(crate) struct MemoryWindow {
    pub start: u64,
    pub end: u64,
    pub device: bool,
}
pub(crate) const IDENTITY_MAP: [MemoryWindow; 2] = [
    MemoryWindow {
        start: 0,
        end: RAM_START,
        device: true,
    },
    MemoryWindow {
        start: RAM_START,
        end: RAM_END,
        device: false,
    },
];
