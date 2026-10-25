//! Echo peers prove caller-local names, endpoint rights and message identity.
use crate::{Error, Handle, abi};

pub(super) fn server() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    crate::notifications::server(input, output);
    let mut buffer = [0; 64];
    for _ in 0..32 {
        assert_eq!(input.receive(&mut buffer).unwrap(), 64);
        // The same local number names our receive right, never the sender's right.
        let foreign = Handle::from_raw(u64::from_le_bytes(buffer[..8].try_into().unwrap()));
        assert_eq!(foreign.send(b"stolen"), Err(Error(abi::DENIED as i64)));
        output.send(&buffer).unwrap();
    }
    assert_eq!(
        input.receive(&mut buffer),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
pub(super) fn client() -> u64 {
    let output = Handle::bootstrap(0).unwrap();
    let input = Handle::bootstrap(1).unwrap();
    crate::notifications::client(input, output);
    endpoint_probe();
    notification_probe();
    assert_eq!(
        Handle::from_raw(0).send(b"invalid"),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert!(Handle::bootstrap(2).is_err());
    assert_eq!(input.send(b"wrong right"), Err(Error(abi::DENIED as i64)));
    assert_eq!(output.revoke(), Err(Error(abi::DENIED as i64)));
    let mut received = [0; 64];
    for round in 0..32 {
        let mut sent = [round; 64];
        sent[..8].copy_from_slice(&output.raw().to_le_bytes());
        output.send(&sent).unwrap();
        assert_eq!(input.receive(&mut received).unwrap(), 64);
        assert_eq!(sent, received);
    }
    output.close().unwrap();
    assert_eq!(output.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
    0
}

// Exercise nonblocking endpoint reception through the architecture's syscall
// entry, including typed refusal without consuming the offered capability.
fn endpoint_probe() {
    use cathedral_user_runtime::ipc;
    let (send, receive) = ipc::channel().unwrap();
    let (offered, peer) = ipc::channel().unwrap();
    assert!(
        matches!(receive.try_receive_with(&mut [0;64]), Err(Error(c)) if c == abi::WOULD_BLOCK as i64)
    );
    assert!(matches!(send.try_receive_with(&mut [0;64]), Err(Error(c)) if c == abi::DENIED as i64));
    send.send_with(b"endpoint", offered).unwrap();
    assert!(
        matches!(receive.try_receive_memory(&mut [0;64]), Err(Error(c)) if c == abi::TOO_SMALL as i64)
    );
    let (_, endpoint) = receive.try_receive_with(&mut [0; 64]).unwrap();
    let endpoint = endpoint.unwrap();
    endpoint.send(b"proof").unwrap();
    assert_eq!(peer.receive(&mut [0; 64]).unwrap(), 5);
    for handle in [send, receive, endpoint, peer] {
        handle.close().unwrap();
    }
    cathedral_user_runtime::write(b"Cathedral ARM64: nonblocking endpoint custody passed\n")
        .unwrap();
}

// Exercise notification escrow through SVC without blocking or accepting another kind.
fn notification_probe() {
    use cathedral_user_runtime::{
        ipc,
        notification::{self, Notification as N, Outcome},
        time,
    };
    let baseline = notification::budget().unwrap();
    let (send, receive) = ipc::channel().unwrap();
    let [wait, signal, control] = N::create().unwrap();
    assert_eq!(
        N::try_accept(receive, &mut [0; 64]),
        Err(Error(abi::WOULD_BLOCK as i64))
    );
    assert_eq!(
        N::try_accept(send, &mut [0; 64]),
        Err(Error(abi::DENIED as i64))
    );
    wait.delegate(send, b"wait").unwrap();
    assert!(
        matches!(receive.try_receive_with(&mut [0; 64]), Err(Error(c)) if c == abi::TOO_SMALL as i64)
    );
    let (len, received) = N::try_accept(receive, &mut [0; 64]).unwrap();
    assert_eq!(len, 4);
    let received = received.unwrap();
    assert_eq!(received.signal(false), Err(Error(abi::DENIED as i64)));
    signal.signal(false).unwrap();
    assert_eq!(
        received.wait(Some(time::now().unwrap())),
        Ok(Outcome::Signaled)
    );
    control.close_object().unwrap();
    for right in [wait, signal, control, received] {
        right.release().unwrap();
    }
    send.close().unwrap();
    receive.close().unwrap();
    assert_eq!(notification::budget().unwrap(), baseline);
    cathedral_user_runtime::write(b"Cathedral ARM64: nonblocking notification custody passed\n")
        .unwrap();
}
