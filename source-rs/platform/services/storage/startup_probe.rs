//! Test-only failures before opening the disk. Argument belongs to the owner;
//! ordinary provider builds never interpret it or contain these paths.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle, task, write};
pub fn run(mode: u64) {
    match mode {
        1 => panic!("deliberate storage startup failure"),
        3 => bad_ready(),
        2 | 4 => {
            let receive = Handle::bootstrap(0).unwrap();
            let send = Handle::bootstrap(1).unwrap();
            write(b"Cathedral storage: startup probe waiting\n").unwrap();
            let mut bytes = [0; 64];
            loop {
                match receive.receive(&mut bytes) {
                    Ok(_) => (),
                    // A shared opener retires malformed setup immediately. This
                    // fixture must exit without opening the disk or panicking.
                    Err(Error(c)) if c == abi::PEER_CLOSED as i64 => task::exit(0),
                    Err(error) => panic!("startup probe receive: {error:?}"),
                }
                if mode == 4 {
                    send.send(b"wrong").unwrap();
                }
            }
        }
        _ => (),
    }
}

/// Complete either selected transport, then deliberately refuse HEALTH. This
/// distinguishes application readiness failure from an invalid setup ACK.
fn bad_ready() -> ! {
    use cathedral_user_runtime::server::{Server, SharedSlot};
    let mut server = Server::open_clients(0).unwrap();
    server.enable_memory();
    static mut SLOT: SharedSlot = SharedSlot::empty();
    let slot = &raw mut SLOT;
    // SAFETY: This startup fixture enters once and exits without opening storage.
    server.enable_shared_control(unsafe { &mut *slot }).unwrap();
    loop {
        match server.next_request() {
            Ok(request) => server.reply(&request, b"wrong").unwrap(),
            Err(Error(c)) if c == abi::PEER_CLOSED as i64 => task::exit(0),
            Err(error) => panic!("startup readiness: {error:?}"),
        }
    }
}
