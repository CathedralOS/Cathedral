// Install the exception entrance and its permanent stack before enabling any IRQ.
// timer.rs owns the fixed-board GICv2 and virtual timer programming.
use super::{BootLayout, context};
use core::{
    arch::asm,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};
pub(super) static IRQ_STACK: AtomicU64 = AtomicU64::new(0);
static FAULT: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
use crate::Fault;
/// # Safety
/// Own EL1 vectors and permanent exception stack; IRQs masked, sole CPU.
pub unsafe fn install_interrupts(layout: &BootLayout, handler: fn(Fault) -> !) {
    IRQ_STACK.store(layout.emergency_tops[3], Ordering::Release);
    FAULT.store(handler as *mut (), Ordering::Release);
    // SAFETY: Vectors are 2-KiB aligned, permanently mapped; permit eagerly saved FP/SIMD.
    unsafe {
        asm!("msr vbar_el1, {}","msr cpacr_el1, {}","isb",in(reg) context::cathedral_vectors as *const (),in(reg) 3u64<<20,options(nostack));
    }
}
/// # Safety
/// Installed exception vectors and callback on the sole CPU.
pub unsafe fn test_breakpoint() {
    // SAFETY: The installed EL1 BRK handler advances the saved PC.
    unsafe {
        asm!("brk #0", options(nostack));
    }
}
pub(super) fn fatal(fault: Fault) -> ! {
    let pointer = FAULT.load(Ordering::Acquire);
    assert!(!pointer.is_null());
    // SAFETY: install_interrupts published exactly this callback type.
    unsafe {
        let callback: fn(Fault) -> ! = core::mem::transmute(pointer);
        callback(fault)
    }
}
