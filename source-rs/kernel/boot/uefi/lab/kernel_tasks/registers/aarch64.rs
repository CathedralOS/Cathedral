//! EL1 sentinels observed across timer preemption, using the architectural counter as deadline.
use core::{
    arch::{asm, global_asm},
    sync::atomic::AtomicU64,
};
global_asm!(include_str!("aarch64.S"));
unsafe extern "C" {
    fn cathedral_register_probe(progress: *const AtomicU64, deadline: u64, pattern: u64) -> u64;
}
/// # Safety
/// Kernel task on the sole EL1 CPU; IRQs enabled, live timer and FP/SIMD initialized.
pub unsafe fn exercise(progress: &AtomicU64, deadline: u64, pattern: u64) -> bool {
    let counter: u64;
    let frequency: u64;
    // SAFETY: Privileged architectural counter reads; no shared state is changed.
    unsafe {
        asm!("mrs {}, cntvct_el0", "mrs {}, cntfrq_el0", out(reg) counter, out(reg) frequency, options(nostack));
    }
    let ticks = deadline.wrapping_sub(cathedral_arch::ticks());
    assert!(ticks < 100);
    let end = counter.wrapping_add(frequency * ticks / 100);
    // SAFETY: Assembly obeys AAPCS64, restores callee/FP state, touches only the supplied atomic counter.
    unsafe { cathedral_register_probe(progress, end, pattern) != 0 }
}
