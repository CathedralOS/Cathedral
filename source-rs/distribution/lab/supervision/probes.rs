use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch},
    yield_now,
};

pub fn bad_wait(child: Child) {
    for (address, length, expected) in [
        (0, 40u64, abi::BAD_ADDRESS),
        (u64::MAX, 40, abi::BAD_ADDRESS),
        (b"readonly".as_ptr() as u64, 40, abi::BAD_ADDRESS),
        (0, 39, abi::INVALID_ARGUMENT),
    ] {
        let result: u64;
        // SAFETY: Integer addresses enter only the kernel's checked-copy boundary;
        // these hostile arguments never become Rust references.
        unsafe {
            core::arch::asm!("int 0x80", inlateout("rax") abi::TASK_WAIT => result,
                in("rdi") child.raw(), in("rsi") address, in("rdx") length,
                options(nostack, preserves_flags));
        }
        assert_eq!(result, expected);
    }
}
pub fn failed_spawn(expected: u64) -> u64 {
    let launch = Launch::bootstrap().unwrap();
    for _ in 0..8 {
        assert_eq!(launch.spawn(0).unwrap_err(), Error(expected as i64));
    }
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}
pub fn passive_peer() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    assert_eq!(input.receive(&mut [0; 64]).unwrap(), 4);
    assert_eq!(
        input.receive(&mut [0; 64]),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
pub fn abandon_child() -> u64 {
    Launch::bootstrap().unwrap().spawn(0).unwrap();
    for _ in 0..32 {
        yield_now();
    }
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}
pub fn blocked_child() -> u64 {
    Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap();
    253 // Must be cancelled when its supervisor exits.
}
pub fn returned_child() -> u64 {
    let child = Launch::bootstrap().unwrap().spawn(u64::MAX).unwrap();
    for _ in 0..32 {
        yield_now();
    }
    bad_wait(child);
    assert_eq!(
        child.wait().unwrap(),
        cathedral_user_runtime::task::Outcome::Returned(u64::MAX)
    );
    assert_eq!(child.wait(), Err(Error(abi::BAD_HANDLE as i64)));
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}
