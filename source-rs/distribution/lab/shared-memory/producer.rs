use super::*;
pub fn run(mode: u64) -> u64 {
    let intro = Handle::bootstrap(0).unwrap();
    let (send, receive) = ipc::channel().unwrap();
    intro.send_with(b"buffer channel", receive).unwrap();
    let mut private = Private::allocate(1).unwrap();
    for (i, byte) in private.bytes_mut().iter_mut().enumerate() {
        *byte = (i % 251) as u8;
    }
    let mut object = private.freeze().unwrap();
    let old = object.handle();
    object.unmap().unwrap();
    assert_eq!(raw::address(old), Err(Error(abi::BAD_ADDRESS as i64)));
    send.send_memory(b"immutable", &object).unwrap();
    if mode == 1 {
        intro.send(b"queued").unwrap();
        // Preserve the live source after failed sends and eventually drain/discard escrow.
        assert!(
            matches!(send.send_memory(b"full", &object), Err(Error(code)) if code == abi::WOULD_BLOCK as i64 || code == abi::PEER_CLOSED as i64)
        );
    }
    drop(object);
    assert_eq!(raw::pages(old), Err(Error(abi::BAD_HANDLE as i64)));
    // Keep the send right until process exit; B waits for that death notification.
    0
}
