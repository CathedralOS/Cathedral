//! Position-independent, embedded ring-3 workload for bring-up. This is not a
//! loader or application SDK; the three syscall numbers follow core/user_tasks/syscall.
use core::arch::global_asm;
global_asm!(include_str!("probe.S"), code = const super::memory::USER_CODE,
    data = const super::memory::USER_DATA, stack = const super::memory::USER_STACK);
unsafe extern "C" {
    static cathedral_user_probe_start: u8;
    static cathedral_user_probe_end: u8;
}
pub fn user_probe() -> &'static [u8] {
    // SAFETY: Assembly labels bound one resident read-only region in this image.
    unsafe {
        let start = &raw const cathedral_user_probe_start;
        let end = &raw const cathedral_user_probe_end;
        core::slice::from_raw_parts(start, (end as usize).checked_sub(start as usize).unwrap())
    }
}
