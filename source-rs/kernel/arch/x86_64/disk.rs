//! Fixed secondary ISA ATA transport. Protocol and persistence live in userspace.
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
pub unsafe fn disk_read(register: u8) -> u8 {
    // SAFETY: Caller owns this fixed controller register.
    unsafe { crate::x86::in8(port(register)) }
}
/// # Safety
/// Same custody as disk_read; caller validates register/value policy.
pub unsafe fn disk_write(register: u8, value: u8) {
    // SAFETY: Caller owns this fixed controller register.
    unsafe {
        crate::x86::out8(port(register), value);
    }
}
/// # Safety
/// Sole controller owner; DRQ set, BSY clear, one 512-byte PIO transfer pending.
pub unsafe fn disk_transfer(bytes: &mut [u8; 512], write: bool) {
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
}
