#![no_std]
#![no_main]
//! Linear GOP display provider. Test fault injection is supplied only by boot.
//! Receives contracts/display.rs packets; surface.rs owns rendering and mapping.
#[cfg(feature = "lab")]
mod probes;
#[cfg(feature = "recovery-lab")]
mod recovery;
mod surface;
use cathedral_contracts::{display as wire, user as abi};
use cathedral_user_runtime::{Error, server::Server};
cathedral_user_runtime::entry!(main);

fn main(_fault_after: u64, _generation: u64) -> u64 {
    #[cfg(feature = "recovery-lab")]
    recovery::startup(_fault_after, _generation);
    #[cfg(feature = "lab")]
    if _fault_after == u64::MAX {
        return probes::run(_generation);
    }
    let mut surface = surface::Surface::open();
    let mut server = Server::open().unwrap();
    #[cfg(feature = "lab")]
    let mut requests = 0;
    loop {
        let request = match server.next_request() {
            Ok(request) => request,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => return 0,
            Err(_) => return 1,
        };
        #[cfg(feature = "recovery-lab")]
        if request.control {
            recovery::request(
                &request.bytes[..request.len],
                cathedral_user_runtime::ipc::Handle::bootstrap(0).unwrap(),
                cathedral_user_runtime::ipc::Handle::bootstrap(1).unwrap(),
            );
        }
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
        let reply = surface.request(&request.bytes[..request.len]);
        server.reply(&request, &wire::encode(reply)).unwrap();
    }
}
