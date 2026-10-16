//! QEMU PC controller transport. Scan-code policy belongs to its user provider.
use crate::board::{DeviceError, InputEvent};
use crate::isa::x86::{in8, out8};
use cathedral_kernel_contracts::user as abi;
/// # Safety
/// Sole boot CPU, IRQs off, exclusive PIC ownership and vector 33 installed.
pub unsafe fn interrupts(enabled: bool) {
    // SAFETY: Only IRQ1's mask changes; timer and other routes retain their state.
    unsafe {
        let mask = in8(0x21);
        out8(0x21, if enabled { mask & !2 } else { mask | 2 });
    }
}
/// # Safety
/// Exclusive controller access on the boot CPU, IRQs off.
pub unsafe fn read() -> Option<InputEvent> {
    // SAFETY: Fixed PC controller registers; inspect ready before consuming data.
    unsafe {
        let status = in8(0x64);
        if status & 1 == 0 {
            return None;
        }
        let byte = in8(0x60);
        Some(if status & 0xc0 != 0 {
            InputEvent::Lost
        } else {
            InputEvent::Byte(
                u16::from(byte)
                    | if status & 0x20 != 0 {
                        abi::KEYBOARD_AUX as u16
                    } else {
                        0
                    },
            )
        })
    }
}
/// # Safety
/// Same controller custody; this adapter validates permitted command/data transactions.
pub unsafe fn write(command: u64, byte: u64) -> Result<(), DeviceError> {
    if !valid_write(command, byte) {
        return Err(DeviceError::InvalidArgument);
    }
    // SAFETY: Never wait in the IRQ-masked kernel. Busy is retried by userspace.
    unsafe {
        if in8(0x64) & 2 != 0 {
            return Err(DeviceError::WouldBlock);
        }
        out8(if command == 1 { 0x64 } else { 0x60 }, byte as u8);
    }
    Ok(())
}

fn valid_write(command: u64, byte: u64) -> bool {
    command <= 1
        && byte <= 255
        && (command == 0 || matches!(byte, 0x20 | 0x60 | 0xa7 | 0xa8 | 0xad | 0xae | 0xd4))
}
#[cfg(test)]
mod tests {
    use super::valid_write;
    #[test]
    fn reset_a20_and_truncated_commands_never_reach_hardware() {
        for byte in 0..=255 {
            assert_eq!(
                valid_write(1, byte),
                matches!(byte, 0x20 | 0x60 | 0xa7 | 0xa8 | 0xad | 0xae | 0xd4)
            );
            assert!(valid_write(0, byte));
        }
        for (command, byte) in [(2, 0), (0, 256), (1, 0x1d4), (u64::MAX, 0)] {
            assert!(!valid_write(command, byte));
        }
    }
}
