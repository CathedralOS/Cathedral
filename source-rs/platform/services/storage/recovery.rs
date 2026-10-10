use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    server::{Request, Server},
    write,
};
pub fn transport() -> Result<(), Error> {
    use cathedral_contracts::user as abi;
    use cathedral_user_runtime::disk;
    for register in [0, 9, u64::MAX] {
        assert_eq!(
            disk::read(register),
            Err(Error(abi::INVALID_ARGUMENT as i64))
        );
    }
    for (register, value) in [(0, 0), (1, 1), (2, 2), (6, 0xf0), (7, 0xc8), (8, 0)] {
        assert_eq!(
            disk::write(register, value),
            Err(Error(abi::INVALID_ARGUMENT as i64))
        );
    }
    for write in [0, 1] {
        for pointer in [0, u64::MAX, 0xffff_8000_0000_1000] {
            assert_eq!(raw_transfer(pointer, write, 512), abi::BAD_ADDRESS);
        }
    }
    // Code is readable, never a writable device destination.
    assert_eq!(
        raw_transfer(transport as *const () as u64, 0, 512),
        abi::BAD_ADDRESS
    );
    for (direction, length) in [(2, 512), (0, 511), (1, 513)] {
        assert_eq!(raw_transfer(0, direction, length), abi::INVALID_ARGUMENT);
    }
    write(b"Cathedral: disk transport checked copies and bounds passed\n")
}
fn raw_transfer(pointer: u64, direction: u64, length: u64) -> u64 {
    let result;
    // SAFETY: Test deliberately supplies hostile syscall arguments; no pointer
    // is dereferenced here. The kernel must reject them before device effects.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") cathedral_contracts::user::DISK_TRANSFER => result,
            in("rdi") pointer, in("rsi") direction, in("rdx") length, options(nostack, preserves_flags));
    }
    result
}
pub fn request(
    bytes: &[u8],
    cut: &mut u8,
    server: &mut Server,
    request: &Request,
) -> Result<bool, Error> {
    if let [0xf1, phase @ 1..=4] = bytes {
        *cut = *phase;
        server.reply(request, b"armed")?;
        return Ok(true);
    }
    if let [0xf0, mode @ 1..=3] = bytes {
        server.reply(request, b"armed")?;
        match mode {
            1 => {
                // SAFETY: Explicit test-only fault confined to this provider.
                unsafe {
                    core::arch::asm!("ud2", options(noreturn));
                }
            }
            3 => loop {
                Handle::bootstrap(0)?.receive(&mut [0; 64])?;
            },
            _ => loop {
                core::hint::spin_loop();
            },
        }
    }
    Ok(false)
}
pub fn checkpoint(phase: u8, requested: u8) {
    if phase == requested {
        let mut message = *b"Cathedral: storage cut=0\n";
        message[23] = b'0' + phase;
        write(&message).unwrap();
        loop {
            core::hint::spin_loop();
        }
    }
}
