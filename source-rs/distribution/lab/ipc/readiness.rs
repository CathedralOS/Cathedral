//! Both-ready precedence, no-consume polling, all-blocked timeout and peer closure.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    time,
};
pub fn receiver() -> u64 {
    let first = Handle::bootstrap(0).unwrap();
    let second = Handle::bootstrap(1).unwrap();
    let ack = Handle::bootstrap(2).unwrap();
    let control = Handle::bootstrap(3).unwrap();
    assert_eq!(
        ipc::wait_two(first, second, time::after(3).unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    for (command, expected, input) in [(b'2', 1, second), (b'1', 0, first)] {
        control.send(&[command]).unwrap();
        assert_eq!(
            ipc::wait_two(first, second, time::after(100).unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            ipc::wait_two(input, control, 0),
            Err(Error(abi::DENIED as i64))
        );
        assert_eq!(ipc::wait_two(first, second, 0).unwrap(), expected);
        let mut bytes = [0; 64];
        assert_eq!(input.receive(&mut bytes).unwrap(), 1);
        assert_eq!(bytes[0], command);
    }
    control.send(b"b").unwrap();
    let mut bytes = [0; 64];
    ack.receive(&mut bytes).unwrap(); // Both incoming messages are now queued.
    assert_eq!(ipc::wait_two(first, second, 0).unwrap(), 0);
    first.receive(&mut bytes).unwrap();
    assert_eq!(ipc::wait_two(first, second, 0).unwrap(), 1);
    second.receive(&mut bytes).unwrap();
    control.send(b"x").unwrap();
    assert_eq!(
        ipc::wait_two(first, second, time::after(100).unwrap()).unwrap(),
        0
    );
    assert_eq!(
        first.receive(&mut bytes),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
pub fn sender() -> u64 {
    let first = Handle::bootstrap(0).unwrap();
    let second = Handle::bootstrap(1).unwrap();
    let ack = Handle::bootstrap(2).unwrap();
    let control = Handle::bootstrap(3).unwrap();
    let mut bytes = [0; 64];
    for output in [second, first] {
        control.receive(&mut bytes).unwrap();
        output.send(&bytes[..1]).unwrap();
    }
    control.receive(&mut bytes).unwrap();
    first.send(b"b").unwrap();
    second.send(b"b").unwrap();
    ack.send(b"ready").unwrap();
    control.receive(&mut bytes).unwrap();
    0
}
