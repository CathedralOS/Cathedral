//! Win64 register sentinels across actual timer preemption.
use core::{arch::global_asm, sync::atomic::AtomicU64};
extern "win64" fn ticks() -> u64 {
    cathedral_arch::ticks()
}
global_asm!(include_str!("x86_64.S"), ticks = sym ticks);
unsafe extern "win64" {
    fn cathedral_register_probe(progress: *const AtomicU64, deadline: u64, pattern: u64) -> u64;
}
/// # Safety
/// Live kernel task on the qemu64 profile, IRQs enabled, timer installed; small positive pattern.
pub unsafe fn exercise(progress: &AtomicU64, deadline: u64, pattern: u64) -> bool {
    // SAFETY: Assembly preserves Win64 state and atomically increments the live counter.
    unsafe { cathedral_register_probe(progress, deadline, pattern) != 0 }
}
