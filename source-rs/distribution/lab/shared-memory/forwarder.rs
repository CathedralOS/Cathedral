use super::*;
pub fn run(mode: u64) -> u64 {
    let input = introduced(Handle::bootstrap(0).unwrap());
    if mode == 1 {
        Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap();
        input.close().unwrap();
        return 0;
    }
    let mut bytes = [0; 64];
    assert!(
        matches!(input.receive_with(&mut bytes), Err(Error(code)) if code == abi::TOO_SMALL as i64)
    );
    assert_eq!(probes::bad_copyout(input), abi::BAD_ADDRESS as i64);
    let (_, object) = input.receive_memory(&mut bytes).unwrap();
    assert_eq!(&bytes[..9], b"immutable");
    assert_eq!(
        input.receive(&mut bytes),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    let object = object.unwrap();
    assert_eq!(
        raw::address(object.handle()),
        Err(Error(abi::BAD_ADDRESS as i64))
    );
    let (send, receive) = ipc::channel().unwrap();
    Handle::bootstrap(1)
        .unwrap()
        .send_with(b"forwarded channel", receive)
        .unwrap();
    send.send_memory(&bytes[..9], &object).unwrap();
    drop(object); // No mapping, no copy, no withdrawal of C's queued capability.
    // C observes process death, not just an explicitly closed channel.
    input.close().unwrap();
    0
}
