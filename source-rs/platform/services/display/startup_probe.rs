//! Test-only pre-initialization failures; absent from ordinary service builds.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle, task};
pub fn run(mode: u64) {
    if mode == 0 {
        return;
    }
    if mode == 1 {
        task::exit(70);
    }
    if mode == 5 {
        bad_ready();
    }
    let receive = Handle::bootstrap(0).unwrap();
    let send = Handle::bootstrap(1).unwrap();
    receive_or_exit(receive);
    if mode == 3 {
        send.send(b"wrong").unwrap();
    }
    loop {
        receive_or_exit(receive);
    }
}
fn receive_or_exit(receive: Handle) {
    match receive.receive(&mut [0; 64]) {
        Ok(_) => (),
        Err(Error(c)) if c == abi::PEER_CLOSED as i64 => task::exit(0),
        Err(error) => panic!("display bootstrap probe: {error:?}"),
    }
}
/// Complete shared setup, then reject INFO without touching the framebuffer.
fn bad_ready() -> ! {
    use cathedral_user_runtime::server::{Server, SharedSlot};
    let mut server = Server::open_clients(0).unwrap();
    server.enable_memory();
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let control = &raw mut CONTROL;
    // SAFETY: One fixture entry lends this slot until process exit.
    server
        .enable_shared_control(unsafe { &mut *control })
        .unwrap();
    loop {
        match server.next_request() {
            Ok(request) => {
                cathedral_user_runtime::write(
                    b"Cathedral display: bootstrap readiness request received\n",
                )
                .unwrap();
                server.reply(&request, b"wrong").unwrap();
            }
            Err(Error(c)) if c == abi::PEER_CLOSED as i64 => task::exit(0),
            Err(error) => panic!("display bootstrap readiness: {error:?}"),
        }
    }
}
