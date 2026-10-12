// QEMU PC selects fixed PS/2 and secondary ATA adapters; timer.rs owns PIC/PIT bring-up.
use super::{Devices, Disk, DiskKind, Keyboard, KeyboardKind};
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
