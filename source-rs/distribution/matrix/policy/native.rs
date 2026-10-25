//! The packaging tool checks the single pure export and embeds its code.
#[cfg(target_os = "none")]
core::arch::global_asm!(include_str!(env!("OMEGA_MATRIX_ASSEMBLY")));
#[cfg(target_os = "none")]
pub(super) fn decide(packet: &mut [u64; 128]) -> u64 {
    unsafe extern "sysv64" {
        fn omega_app_step(packet: *mut u64) -> u64;
    }
    // SAFETY: This built-in export accepts exactly one exclusive 128-word
    // packet, retains no pointers and has no external dependencies or state.
    unsafe { omega_app_step(packet.as_mut_ptr()) }
}
#[cfg(all(test, windows))]
pub(super) fn decide(packet: &mut [u64; 128]) -> u64 {
    crate::tests::decide(packet)
}

#[cfg(not(any(target_os = "none", all(test, windows))))]
pub(super) fn decide(_: &mut [u64; 128]) -> u64 {
    panic!("matrix policy runs in the x86-64 guest or Windows conformance fixture")
}
