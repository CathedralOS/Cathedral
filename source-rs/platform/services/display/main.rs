#![no_std]
#![no_main]
//! Linear GOP display provider. Test fault injection is supplied only by boot.
#[cfg(feature = "lab")]
mod probes;
mod surface;
use cathedral_contracts::{display as wire, user as abi};
use cathedral_user_runtime::{Error, ipc::Handle};
cathedral_user_runtime::entry!(main);

fn main(_fault_after: u64, _generation: u64) -> u64 {
    #[cfg(feature = "lab")]
    if _fault_after == u64::MAX {
        return probes::run(_generation);
    }
    let mut surface = surface::Surface::open();
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    #[cfg(feature = "lab")]
    let mut requests = 0;
    loop {
        let mut bytes = [0; 64];
        let size = match input.receive(&mut bytes) {
            Ok(size) => size,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => return 0,
            Err(_) => return 1,
        };
        #[cfg(feature = "lab")]
        {
            requests += 1;
            if _fault_after != 0 && _generation == 0 && requests == _fault_after {
                // SAFETY: Explicit boot-controlled fault injection for recovery testing.
                unsafe {
                    core::arch::asm!("ud2", options(noreturn));
                }
            }
        }
        let reply = surface.request(&bytes[..size]);
        output.send(&wire::encode(reply)).unwrap();
    }
}
