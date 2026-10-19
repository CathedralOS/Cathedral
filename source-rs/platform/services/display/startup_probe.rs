//! Test-only pre-initialization failures; absent from ordinary service builds.
use cathedral_user_runtime::task;
pub fn run(mode: u64, environment: &cathedral_user_runtime::task::Environment) {
    if mode == 0 {
        return;
    }
    if mode == 1 {
        task::exit(70);
    }
    let receive = environment.endpoint(0).unwrap();
    let send = environment.endpoint(1).unwrap();
    receive.receive(&mut [0; 64]).unwrap();
    if mode == 3 {
        send.send(b"wrong").unwrap();
    }
    loop {
        receive.receive(&mut [0; 64]).unwrap();
    }
}
