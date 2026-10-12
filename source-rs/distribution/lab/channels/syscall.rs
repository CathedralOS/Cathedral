//! Raw trap fixture for hostile pointer/copyout probes; no Rust references to supplied addresses.
pub(super) fn call(number: u64, a: u64, b: u64, c: u64) -> u64 {
    let result;
    // SAFETY: Integers only, deliberately invalid pointers reach checked kernel copies.
    // No Rust references are made from them; the trap preserves non-result registers.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") number => result, in("rdi") a, in("rsi") b, in("rdx") c, options(nostack, preserves_flags));
    }
    result
}

#[repr(align(128))]
pub(super) struct Packet(pub [u8; cathedral_contracts::user::IPC_PACKET_BYTES]);
