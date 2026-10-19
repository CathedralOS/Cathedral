//! Test-only pre-initialization failures; absent from ordinary service builds.
use cathedral_user_runtime::{ipc::Handle, task};
pub fn run(mode: u64) {
    if mode == 0 {
        return;
    }
    if mode == 1 {
        task::exit(70);
    }
    let receive = Handle::bootstrap(0).ok().unwrap();
    let send = Handle::bootstrap(1).ok().unwrap();
    receive.receive(&mut [0; 64]).unwrap();
    if mode == 3 {
        send.send(b"wrong").unwrap();
    }
    loop {
        receive.receive(&mut [0; 64]).unwrap();
    }
}
