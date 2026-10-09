//! Deliberately hostile raw ABI probes; normal programs use the platform wrappers.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle};
use core::arch::asm;

pub fn fault() -> ! {
    // SAFETY: Intentional contained user fault to test endpoint teardown.
    unsafe {
        asm!("ud2", options(noreturn));
    }
}
fn raw(number: u64, handle: Handle, address: u64, size: u64) -> u64 {
    let result;
    // SAFETY: Deliberately invalid integer addresses passed only to the kernel's
    // checked copy boundary. No Rust reference is constructed from those addresses.
    unsafe {
        asm!("int 0x80", inlateout("rax") number => result,
            in("rdi") handle.raw(), in("rsi") address, in("rdx") size,
            options(nostack, preserves_flags));
    }
    result
}
pub fn sender() -> u64 {
    let output = Handle::bootstrap(0).unwrap();
    let signal = Handle::bootstrap(1).unwrap();
    output.send(b"retained message").unwrap();
    assert_eq!(
        output.send(b"queue full"),
        Err(Error(abi::WOULD_BLOCK as i64))
    );
    assert_eq!(raw(abi::IPC_SEND, output, u64::MAX, 8), abi::BAD_ADDRESS);
    assert_eq!(raw(abi::IPC_SEND, output, 0, 65), abi::INVALID_ARGUMENT);
    signal.send(b"ready").unwrap();
    0
}
pub fn receiver() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let signal = Handle::bootstrap(1).unwrap();
    assert_eq!(signal.receive(&mut [0; 64]).unwrap(), 5);
    let readonly = b"read only destination";
    for address in [
        0,
        u64::MAX,
        0x0000_8000_0000_0000,
        0x0000_0080_0020_4000,
        readonly.as_ptr() as u64,
    ] {
        assert_eq!(raw(abi::IPC_RECEIVE, input, address, 8), abi::BAD_ADDRESS);
    }
    // Mapped writable page, but the complete buffer crosses into the stack guard.
    assert_eq!(
        raw(abi::IPC_RECEIVE, input, 0x0000_0080_0020_3ffc, 8),
        abi::BAD_ADDRESS
    );
    assert_eq!(raw(abi::IPC_RECEIVE, input, 0, 65), abi::INVALID_ARGUMENT);
    assert_eq!(
        input.receive(&mut [0; 2]),
        Err(Error(abi::TOO_SMALL as i64))
    );
    let mut buffer = [0; 64];
    let length = input.receive(&mut buffer).unwrap();
    assert_eq!(&buffer[..length], b"retained message");
    assert_eq!(
        input.receive(&mut buffer),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
