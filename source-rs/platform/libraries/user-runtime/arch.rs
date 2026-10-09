//! CPU-specific ABI plumbing; platform code imports only shared contracts.
#[cfg(target_arch = "x86_64")]
use core::arch::{asm, global_asm};

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
global_asm!(include_str!("x86_64.S"));

#[cfg(target_arch = "x86_64")]
pub(super) fn call(number: u64, first: u64, second: u64) -> u64 {
    call3(number, first, second, 0)
}
pub(super) fn call3(number: u64, first: u64, second: u64, third: u64) -> u64 {
    let result;
    // SAFETY: This library runs only in a Cathedral lab task. The installed gate
    // preserves every register except RAX and copies any write buffer before
    // returning. Compiler memory effects are retained across the boundary.
    unsafe {
        asm!("int 0x80", inlateout("rax") number => result, in("rdi") first,
            in("rsi") second, in("rdx") third, options(nostack, preserves_flags));
    }
    result
}
pub(super) fn stop() -> ! {
    // SAFETY: A failed non-returning exit must terminate through the user-fault path.
    unsafe {
        asm!("ud2", options(noreturn));
    }
}
#[cfg(not(target_arch = "x86_64"))]
compile_error!("Only the x86-64 user ABI is implemented");
