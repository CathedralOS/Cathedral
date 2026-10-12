// QEMU PC selects fixed PS/2 and secondary ATA adapters; timer.rs owns PIC/PIT bring-up.
use super::{Devices, Disk, DiskKind, Keyboard, KeyboardKind};
/// Selected board's bootstrap UART window and privileged port operations.
pub struct DiagnosticUart {
    pub base: u16,
    pub read: unsafe fn(u16) -> u8,
    pub write: unsafe fn(u16, u8),
}
pub fn diagnostic_uart() -> DiagnosticUart {
    DiagnosticUart {
        base: 0x3f8,
        read: crate::isa::x86::in8,
        write: crate::isa::x86::out8,
    }
}
pub(super) fn devices() -> Devices {
    Devices {
        keyboard: Some(Keyboard {
            kind: KeyboardKind::Ps2,
        }),
        disk: Some(Disk {
            kind: DiskKind::Ata,
        }),
    }
}
