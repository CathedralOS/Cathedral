#![no_std]
#![no_main]
//! Distribution-owned test pattern and recovery policy.
mod client;
mod probes;
mod supervisor;
use cathedral_contracts::{display, user as abi};
use cathedral_user_runtime::{Error, ipc::Handle, time};
cathedral_user_runtime::entry!(main);
fn main(role: u64, _: u64) -> u64 {
    assert_eq!(
        cathedral_user_runtime::display::mapping(),
        Err(Error(abi::DENIED as i64))
    );
    match role {
        0 => supervisor::run(),
        1 => client::run(),
        2 => observer(),
        4 => probes::owner(),
        5 => probes::failed_admission(),
        6 => probes::peer(),
        3 => {
            // SAFETY: Deliberately probe an ungranted virtual device address.
            // Assembly makes no Rust reference; the expected user fault is contained.
            unsafe {
                core::arch::asm!("mov rax, [rax]", "ud2", in("rax") display::USER_ADDRESS, options(noreturn));
            }
        }
        _ => 254,
    }
}
fn observer() -> u64 {
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
