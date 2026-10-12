//! EL0 entry and SVC ABI: x8 number, x0/x1/x2 arguments, x0 result.
use core::arch::{asm, global_asm};
global_asm!(
    ".section .text._start,\"ax\"",
    ".global _start",
    ".type _start,%function",
    "_start:",
    "bl cathedral_user_main",
    "brk #0"
);
pub(crate) fn call(number: u64, first: u64, second: u64) -> u64 {
    call3(number, first, second, 0)
}
pub(crate) fn call3(number: u64, first: u64, second: u64, third: u64) -> u64 {
    let result;
    // SAFETY: Cathedral's installed SVC vector preserves everything except the x0 result.
    // Memory effects remain visible across the synchronous kernel boundary.
    unsafe {
        asm!("svc #0",in("x8") number,inlateout("x0") first=>result,in("x1") second,in("x2") third,options(nostack));
    }
    result
}
pub(crate) fn stop() -> ! {
    // SAFETY: Terminate a failed non-returning exit through the contained user-fault path.
    unsafe {
        asm!("brk #0", options(noreturn));
    }
}
