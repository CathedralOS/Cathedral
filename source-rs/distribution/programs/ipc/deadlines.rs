//! Deadline completion, authority and retained-message integration probes.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle, time, yield_now};

fn raw(input: Handle, address: u64, deadline: u64) -> u64 {
    let result;
    // SAFETY: Hostile integer addresses go only through the kernel checked-copy ABI.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") abi::IPC_RECEIVE_UNTIL => result,
            in("rdi") input.raw(), in("rsi") address, in("rdx") deadline,
            options(nostack, preserves_flags));
    }
    result
}
pub fn receiver(revoke: u64) -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let mut bytes = [0xa5; 64];
    for address in [
        0,
        u64::MAX,
        b"readonly".as_ptr() as u64,
        0x0000_0080_0020_3fe0,
    ] {
        assert_eq!(raw(input, address, 0), abi::BAD_ADDRESS);
    }
    assert_eq!(
        output.receive_until(&mut bytes, 0),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        Handle::from_raw(0).receive_until(&mut bytes, 0),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert_eq!(
        input.receive_until(&mut bytes, 0),
        Err(Error(abi::TIMED_OUT as i64))
    );
    // Both tasks are blocked here; a timer must retire only this pending receive.
    assert_eq!(
        input.receive_until(&mut bytes, time::after(3).unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    output.send(b"late").unwrap();
    for _ in 0..32 {
        yield_now();
    }
    assert_eq!(bytes, [0xa5; 64]); // No delayed completion wrote to the old destination.
    assert_eq!(raw(input, 0, 0), abi::BAD_ADDRESS); // Queued reply must survive bad copyout.
    assert_eq!(input.receive_until(&mut bytes, 0).unwrap(), 4); // Ready beats expiry.
    assert_eq!(&bytes[..4], b"late");
    output.send(b"stop").unwrap();
    for _ in 0..32 {
        yield_now();
    }
    let error = if revoke != 0 {
        abi::REVOKED
    } else {
        abi::PEER_CLOSED
    };
    assert_eq!(input.receive_until(&mut bytes, 0), Err(Error(error as i64)));
    0
}
pub fn sender(revoke: u64) -> u64 {
    let output = Handle::bootstrap(0).unwrap();
    let revoker = Handle::bootstrap(1).unwrap();
    let control = Handle::bootstrap(2).unwrap();
    assert_eq!(control.receive(&mut [0; 64]).unwrap(), 4);
    output.send(b"late").unwrap();
    assert_eq!(control.receive(&mut [0; 64]).unwrap(), 4);
    if revoke != 0 {
        revoker.revoke().unwrap();
        assert_eq!(
            control.receive(&mut [0; 64]),
            Err(Error(abi::PEER_CLOSED as i64))
        );
    }
    0
}
pub fn denied() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    assert_eq!(
        input.receive_until(&mut [0; 64], 0),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        cathedral_user_runtime::keyboard::read_until(0),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        input.receive(&mut [0; 64]),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
