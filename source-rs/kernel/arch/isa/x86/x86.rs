// Instructions shared by the x86 family; the active backend is x86-64.

use core::arch::asm;

/// # Safety
/// Caller must run with I/O privilege and own access to this port.
pub unsafe fn out8(port: u16, value: u8) {
    // SAFETY: I/O privilege and device access are caller obligations.
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

/// # Safety
/// Caller must run with I/O privilege and own access to this port.
pub unsafe fn in8(port: u16) -> u8 {
    let value: u8;
    // SAFETY: I/O privilege and device access are caller obligations.
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// # Safety
/// Caller must run privileged and own the CPU's interrupt-enable policy.
pub unsafe fn disable_interrupts() {
    // SAFETY: Privilege and interrupt policy are caller obligations.
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

/// # Safety
/// Caller must own this CPU and accept that execution never resumes normally.
pub unsafe fn halt_forever() -> ! {
    // SAFETY: Caller transfers the CPU into terminal idle.
    unsafe {
        disable_interrupts();
    }
    loop {
        // SAFETY: The caller guarantees privileged execution.
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}
