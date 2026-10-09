//! QEMU PC controller transport. Scan-code policy belongs to its user provider.
use super::{in8, out8};
/// # Safety
/// Sole boot CPU, IRQs off, exclusive PIC ownership and vector 33 installed.
pub unsafe fn keyboard_irq(enabled: bool) {
    // SAFETY: Only IRQ1's mask changes; timer and other routes retain their state.
    unsafe {
        let mask = in8(0x21);
        out8(0x21, if enabled { mask & !2 } else { mask | 2 });
    }
}
/// # Safety
/// Exclusive controller access on the boot CPU, IRQs off.
pub unsafe fn keyboard_read() -> Option<(u8, u8)> {
    // SAFETY: Fixed PC controller registers; inspect ready before consuming data.
    unsafe {
        let status = in8(0x64);
        (status & 1 != 0).then(|| (status, in8(0x60)))
    }
}
/// # Safety
/// Same controller custody; caller validates permitted command/data transactions.
pub unsafe fn keyboard_write(command: bool, byte: u8) -> bool {
    // SAFETY: Never wait in the IRQ-masked kernel. Busy is retried by userspace.
    unsafe {
        if in8(0x64) & 2 != 0 {
            return false;
        }
        out8(if command { 0x64 } else { 0x60 }, byte);
    }
    true
}
