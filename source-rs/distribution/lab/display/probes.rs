//! Device denial, failed display admission and independent observer progress.
use cathedral_contracts::{display, user as abi};
use cathedral_user_runtime::time;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Launch, Outcome},
};
pub fn owner() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    for mode in 0..4 {
        let result = launch.spawn(mode).unwrap().wait().unwrap();
        match (mode, result) {
            (0, Outcome::Returned(0)) => {}
            (
                1,
                Outcome::Fault {
                    vector: 14,
                    error: 0x15,
                    address: display::USER_ADDRESS,
                    ..
                },
            ) => {}
            (
                2 | 3,
                Outcome::Fault {
                    vector: 14,
                    error: 4,
                    ..
                },
            ) => {}
            _ => panic!("unexpected framebuffer probe outcome"),
        }
    }
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}
pub fn failed_admission() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    for _ in 0..2 {
        assert_eq!(launch.spawn(0).unwrap_err(), Error(abi::NO_MEMORY as i64));
    }
    Handle::bootstrap(0).unwrap().send(b"done").unwrap();
    0
}
pub fn peer() -> u64 {
    assert_eq!(
        Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap(),
        4
    );
    0
}

pub fn observer() -> u64 {
    let command = Handle::bootstrap(0).unwrap();
    let report = Handle::bootstrap(1).unwrap();
    for _ in 0..2 {
        let mut bytes = [0; 64];
        assert_eq!(command.receive(&mut bytes).unwrap(), 8);
        let deadline = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        let start = time::now().unwrap();
        assert!(!time::reached(start, deadline));
        report.send(&start.to_le_bytes()).unwrap();
        let mut samples = 0u64;
        while !time::reached(time::now().unwrap(), deadline) {
            samples += 1;
        }
        assert!(samples > 1);
        report.send(&samples.to_le_bytes()).unwrap();
    }
    0
}

pub fn unmapped() -> ! {
    // SAFETY: Deliberately probe an ungranted virtual device address.
    // Assembly makes no Rust reference; the expected user fault is contained.
    unsafe {
        core::arch::asm!("mov rax, [rax]", "ud2", in("rax") display::USER_ADDRESS, options(noreturn));
    }
}
