//! Fixed secondary ISA ATA transport. Protocol and persistence live in userspace.
use crate::board::DeviceError;
use core::arch::asm;
fn port(register: u8) -> u16 {
    assert!((1..=8).contains(&register));
    if register == 8 {
        0x376
    } else {
        0x170 + u16::from(register)
    }
}
/// # Safety
/// Exclusive boot-approved controller custody, sole CPU, privileged execution.
pub unsafe fn read(register: u64) -> Result<u8, DeviceError> {
    if !(1..=8).contains(&register) {
        return Err(DeviceError::InvalidArgument);
    }
    // SAFETY: Caller owns this fixed controller register.
    Ok(unsafe { crate::isa::x86::in8(port(register as u8)) })
}
/// # Safety
/// Same custody as read; this adapter validates register/value restrictions.
pub unsafe fn write(register: u64, value: u64) -> Result<(), DeviceError> {
    if !valid_write(register, value) {
        return Err(DeviceError::InvalidArgument);
    }
    // SAFETY: Caller owns this fixed controller register.
    unsafe {
        crate::isa::x86::out8(port(register as u8), value as u8);
    }
    Ok(())
}
/// # Safety
/// Sole controller owner; one bounded PIO transfer requested. Readiness is checked here.
pub unsafe fn transfer(bytes: &mut [u8; 512], write: bool) -> Result<(), DeviceError> {
    // SAFETY: Exclusive controller; non-clearing status read before bounded PIO transfer.
    if unsafe { read(8)? } & 0xa9 != 8 {
        return Err(DeviceError::WouldBlock);
    }
    for pair in bytes.chunks_exact_mut(2) {
        let mut value = u16::from_le_bytes([pair[0], pair[1]]);
        // SAFETY: Fixed owned data port and bounded one-sector transfer.
        unsafe {
            if write {
                asm!("out dx, ax", in("dx") 0x170u16, in("ax") value, options(nomem, nostack, preserves_flags));
            } else {
                asm!("in ax, dx", in("dx") 0x170u16, out("ax") value, options(nomem, nostack, preserves_flags));
            }
        }
        if !write {
            pair.copy_from_slice(&value.to_le_bytes());
        }
    }
    Ok(())
}

fn valid_write(register: u64, value: u64) -> bool {
    match register {
        2 => value == 1,
        3..=5 => value <= 255,
        6 => (0xe0..=0xef).contains(&value),
        7 => matches!(value, 0x20 | 0x30 | 0xe7 | 0xec),
        8 => matches!(value, 2 | 6),
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::valid_write;
    #[test]
    fn bounded_pio_whitelist_excludes_dma_multisector_and_other_devices() {
        for (register, value) in [
            (2, 1),
            (3, 255),
            (6, 0xef),
            (7, 0x20),
            (7, 0x30),
            (7, 0xe7),
            (7, 0xec),
            (8, 6),
        ] {
            assert!(valid_write(register, value));
        }
        for (register, value) in [
            (0, 0),
            (1, 0),
            (9, 0),
            (2, 2),
            (3, 256),
            (6, 0xf0),
            (7, 0xc8),
            (7, 0xca),
            (7, 0x120),
            (8, 0),
            (u64::MAX, 1),
        ] {
            assert!(!valid_write(register, value));
        }
    }
}
