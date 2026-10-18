//! Test-only failures before opening the disk. Argument belongs to the owner;
//! ordinary provider builds never interpret it or contain these paths.
use cathedral_user_runtime::{ipc::Handle, write};
pub fn run(mode: u64) {
    match mode {
        1 => panic!("deliberate storage startup failure"),
        2 | 3 => {
            let receive = Handle::bootstrap(0).unwrap();
            let send = Handle::bootstrap(1).unwrap();
            write(b"Cathedral storage: startup probe waiting\n").unwrap();
            let mut bytes = [0; 64];
            loop {
                receive.receive(&mut bytes).unwrap();
                if mode == 3 {
                    send.send(b"wrong").unwrap();
                }
            }
        }
        _ => (),
    }
}
