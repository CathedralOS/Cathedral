use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    task::{Launch, Outcome, Port},
    write, yield_now,
};
pub fn run() -> u64 {
    let provider = Launch::at(0).unwrap().spawn(0).unwrap();
    let first = Launch::at(1).unwrap().spawn(0).unwrap().wait().unwrap();
    let second = Launch::at(2).unwrap().spawn(1).unwrap().wait().unwrap();
    assert_eq!(first, second);
    let first_marker = if cfg!(feature = "retention") { 9 } else { 1 };
    let final_marker = if cfg!(feature = "retention") { 26 } else { 2 };
    if cfg!(feature = "retention") && first == Outcome::Returned(25) {
        write(b"Cathedral objects: collected revision 25\n").unwrap();
    } else if first == Outcome::Returned(first_marker) {
        #[cfg(feature = "retention")]
        write(b"Cathedral objects: pinned revision 9\n").unwrap();
        #[cfg(not(feature = "retention"))]
        write(b"Cathedral objects: acknowledged revision 1\n").unwrap();
        // Host kills QEMU here, with the provider's committed database still open.
    } else {
        assert_eq!(first, Outcome::Returned(final_marker));
        let (send, receive) = Port::at(0).unwrap().connect().unwrap();
        send.send(
            &wire::Request {
                op: wire::CLOSE,
                object: 0,
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
