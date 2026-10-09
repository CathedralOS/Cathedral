use cathedral_contracts::{display, user as abi};
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
