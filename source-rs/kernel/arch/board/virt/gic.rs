//! Fixed-board GICv2 routing. Architectural counter/rearm instructions live in the CPU backend.
use super::{GIC_CPU as C, GIC_DISTRIBUTOR as D, TIMER_IRQ};
/// # Safety
/// Exclusive single-CPU GIC ownership, IRQs masked and exception vectors installed.
pub(crate) unsafe fn start_timer_route() {
    // SAFETY: Reserved Device-mapped controller; mask/clear inherited routes before enabling timer only.
    unsafe {
        (D as *mut u32).write_volatile(0);
        let banks = (((D + 4) as *const u32).read_volatile() & 31) + 1;
        for bank in 0..banks as usize {
            ((D + 0x180 + bank * 4) as *mut u32).write_volatile(u32::MAX);
            ((D + 0x280 + bank * 4) as *mut u32).write_volatile(u32::MAX);
        }
        let group = (D + 0x80) as *mut u32;
        group.write_volatile(group.read_volatile() & !(1 << TIMER_IRQ));
        ((D + 0x400 + TIMER_IRQ as usize) as *mut u8).write_volatile(0x80);
        ((D + 0x100) as *mut u32).write_volatile(1 << TIMER_IRQ);
        ((C + 4) as *mut u32).write_volatile(0xff);
        ((C + 8) as *mut u32).write_volatile(0);
        (C as *mut u32).write_volatile(1);
        (D as *mut u32).write_volatile(1);
        core::arch::asm!("dsb sy", options(nostack));
    }
}
/// # Safety
/// IRQ exception on sole CPU; exclusive GIC custody.
pub(crate) unsafe fn acknowledge() -> u32 {
    // SAFETY: Read the current controller interrupt token exactly once.
    unsafe { ((C + 0xc) as *const u32).read_volatile() }
}
/// # Safety
/// Token came from acknowledge; source rearmed/cleared before EOI.
pub(crate) unsafe fn complete(token: u32) {
    // SAFETY: Retire the active token on its originating CPU interface.
    unsafe {
        ((C + 0x10) as *mut u32).write_volatile(token);
    }
}
