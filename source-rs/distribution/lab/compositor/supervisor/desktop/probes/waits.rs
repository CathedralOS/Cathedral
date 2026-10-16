//! Exercise the syscall boundary, including copied arrays and later ready receivers.
use super::*;
use cathedral_user_runtime::ipc;
pub(super) fn check(display: Pair, app: Pair) {
    let deadline = time::now().unwrap();
    assert_eq!(raw(u64::MAX, 1, deadline), abi::BAD_ADDRESS);
    assert_eq!(raw(0, 0, deadline), abi::INVALID_ARGUMENT);
    assert_eq!(raw(0, 9, deadline), abi::INVALID_ARGUMENT);
    #[repr(align(64))]
    struct Words([u64; 8]);
    let words = Words([display.1.raw(); 8]);
    assert_eq!(raw(words.0.as_ptr() as u64, 8, deadline), abi::TIMED_OUT);
    app.0
        .send(&cathedral_platform_contracts::display::encode([
            20, 0, 0, 0, 0, 0,
        ]))
        .unwrap();
    let handles = [display.1, display.1, display.1, display.1, app.1];
    assert_eq!(ipc::wait_many(&handles, time::after(300).unwrap()), Ok(4));
    assert_eq!(
        ipc::wait_many(&[app.1, display.0], time::now().unwrap()),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(ipc::wait_many(&handles, time::now().unwrap()), Ok(4));
    assert_eq!(app.1.receive(&mut [0; 64]).unwrap(), 48);
    marker(b"Cathedral session: wait-set copy bounds complete authority validation later readiness and non-consumption passed\n");
}
fn raw(address: u64, count: u64, deadline: u64) -> u64 {
    let result;
    // SAFETY: Lab-only ABI probes. The kernel must validate these addresses before
    // copying; the installed gate preserves all registers except the result.
    unsafe {
        core::arch::asm!("int 0x80",inlateout("rax") abi::IPC_WAIT_MANY=>result,in("rdi") address,in("rsi") count,in("rdx") deadline,options(nostack,preserves_flags));
    }
    result
}
