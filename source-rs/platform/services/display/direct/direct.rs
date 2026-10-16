// Immediate display protocol: receive, validate and draw through the shared surface.
use crate::surface;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display as wire;
use cathedral_user_runtime::{Error, server::Server};
pub fn run(_fault_after: u64, _generation: u64) -> u64 {
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
        let bytes = &request.bytes[..request.len];
        let reply =
            if request.control && wire::decode(bytes).is_some_and(|words| words[0] == wire::BLIT) {
                [abi::DENIED, 0, 0, 0, 0, 0]
            } else {
                surface.request(bytes)
            };
        server.reply(&request, &wire::encode(reply)).unwrap();
    }
}
