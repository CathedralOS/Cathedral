use cathedral_platform_contracts::database as wire;
use cathedral_user_runtime::{
    task::{Launch, Outcome, Port},
    write, yield_now,
};
pub fn run() -> u64 {
    let provider = Launch::at(0).unwrap().spawn(0).unwrap();
    let first = Launch::at(1).unwrap().spawn(0).unwrap().wait().unwrap();
    let second = Launch::at(2).unwrap().spawn(1).unwrap().wait().unwrap();
    assert_eq!(first, second);
    if first == Outcome::Returned(1) {
        write(b"Cathedral objects: acknowledged revision 1\n").unwrap();
        // Host kills QEMU here, with the provider's committed database still open.
    } else {
        assert_eq!(first, Outcome::Returned(2));
        let (send, receive) = Port::at(0).unwrap().connect().unwrap();
        send.send(
            &wire::Request {
                op: wire::CLOSE,
                expected: 0,
                offset: 0,
                len: 0,
            }
            .encode(),
        )
        .unwrap();
        assert_eq!(receive.receive(&mut [0; 64]).unwrap(), 64);
        assert_eq!(provider.wait().unwrap(), Outcome::Returned(0));
        write(b"Cathedral objects: recovered content and private roots passed\n").unwrap();
    }
    loop {
        yield_now();
    }
}
