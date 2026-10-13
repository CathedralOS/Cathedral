//! Invalid memory-attachment copyout must leave the queued capability untouched.
use super::*;
pub fn bad_copyout(input: Handle) -> i64 {
    let result: u64;
    // SAFETY: Deliberately invalid user destination. Only the kernel touches it,
    // through its checked-copy boundary; the lab expects BAD_ADDRESS.
    unsafe {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("int 0x80", inlateout("rax") abi::IPC_RECEIVE_MEMORY => result,
            in("rdi") input.raw(), in("rsi") 0u64, in("rdx") 0u64, options(nostack));
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("svc #0", in("x8") abi::IPC_RECEIVE_MEMORY, inlateout("x0") input.raw() => result,
            in("x1") 0u64, in("x2") 0u64, options(nostack));
    }
    result as i64
}
