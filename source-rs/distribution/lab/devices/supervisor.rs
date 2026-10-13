//! Restart policy belongs to userspace; a launch right is not register authority.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    device::Device,
    task::{Launch, Outcome, Port},
    time,
};
pub fn run() -> u64 {
    assert_eq!(Device::at(0).unwrap_err(), Error(abi::BAD_HANDLE as i64));
    let launch = Launch::bootstrap().unwrap();
    let port = Port::bootstrap().unwrap();
    let mut previous = 0;
    for mode in [0, 1, 2, 1, 0] {
        let child = launch.spawn(previous).unwrap();
        let (requests, replies) = port.connect().unwrap();
        let mut message = [0; 64];
        assert_eq!(replies.receive(&mut message).unwrap(), 8);
        let name = u64::from_le_bytes(message[..8].try_into().unwrap());
        assert_ne!(name, previous);
        assert_eq!(
            Device::from_raw(name).read8(0),
            Err(Error(abi::BAD_HANDLE as i64))
        );
        requests.send(&[mode]).unwrap();
        if mode == 2 {
            assert_eq!(replies.receive(&mut message).unwrap(), 7);
            let until = time::after(2).unwrap();
            while !time::reached(time::now().unwrap(), until) {
                cathedral_user_runtime::yield_now();
            }
            child.cancel().unwrap();
            assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
        } else {
            assert_eq!(replies.receive(&mut message).unwrap(), 8);
            if mode == 0 {
                requests.send(b"stop").unwrap();
            }
            let outcome = child.wait().unwrap();
            if mode == 1 {
                assert!(matches!(outcome, Outcome::Fault { .. }));
            } else {
                assert_eq!(outcome, Outcome::Returned(0));
            }
        }
        previous = name;
    }
    cathedral_user_runtime::write(
        b"device grant isolation, IRQ loopback, fault and blocked cancellation passed\n",
    )
    .unwrap();
    cathedral_user_runtime::write(b"combined IPC, device IRQ and deadline waits passed\n").unwrap();
    0
}
