use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Outcome},
    time, yield_now,
};

fn raw(number: u64, first: u64, second: u64, third: u64) -> u64 {
    let result;
    // SAFETY: Hostile integers enter checked kernel copy/authority boundaries;
    // no invalid Rust reference is constructed.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") number => result,
        in("rdi") first, in("rsi") second, in("rdx") third, options(nostack, preserves_flags));
    }
    result
}
pub fn clock() {
    let ticket = raw(abi::CLOCK_HANDLE, 0, 0, 0);
    assert!((ticket as i64) > 0);
    assert_eq!(raw(abi::CLOCK_READ, 0, 0, 8), abi::BAD_HANDLE);
    for address in [0, u64::MAX, b"readonly".as_ptr() as u64] {
        assert_eq!(raw(abi::CLOCK_READ, ticket, address, 8), abi::BAD_ADDRESS);
    }
    assert_eq!(raw(abi::CLOCK_READ, ticket, 0, 7), abi::INVALID_ARGUMENT);
    assert_eq!(
        time::after(1 << 63),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    assert_eq!(
        Child::from_raw(0).cancel(),
        Err(Error(abi::BAD_HANDLE as i64))
    );
}
pub fn completed() -> u64 {
    clock();
    let launch = Launch::bootstrap().unwrap();
    let child = launch.spawn(u64::MAX).unwrap();
    let deadline = time::after(100).unwrap();
    assert_eq!(
        child.wait_until(deadline).unwrap(),
        Outcome::Returned(u64::MAX)
    );
    assert_eq!(
        child.wait_until(deadline),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    let child = launch.spawn(42).unwrap();
    for _ in 0..32 {
        yield_now();
    }
    child.cancel().unwrap(); // A completed child's return status must not become Cancelled.
    let past = time::now().unwrap().wrapping_sub(1);
    assert_eq!(child.wait_until(past).unwrap(), Outcome::Returned(42));
    assert_eq!(child.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}

pub fn idle() -> u64 {
    let child = Launch::bootstrap().unwrap().spawn(0).unwrap();
    // Both peers block in receive. The boot CPU must idle and wake on the timer.
    let deadline = time::after(12).unwrap();
    assert_eq!(
        child.wait_until(deadline),
        Err(Error(abi::TIMED_OUT as i64))
    );
    assert!(time::reached(time::now().unwrap(), deadline));
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}
