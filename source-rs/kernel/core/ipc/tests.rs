use super::*;
fn pair(epoch: u64) -> Ipc {
    Ipc::new(
        epoch,
        3,
        &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: Some(2),
        }],
    )
    .unwrap()
}
#[test]
fn tickets_are_bound_to_caller_epoch_and_right() {
    let mut ipc = pair(1);
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    assert_eq!(ipc.send(1, send, b"stolen"), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.send(1, receive, b"wrong right"), Err(abi::DENIED));
    assert_eq!(ipc.revoke(0, send), Err(abi::DENIED));
    for bad in [0, u64::MAX, send + 1, send ^ (1 << 8)] {
        assert_eq!(ipc.send(0, bad, b"bad"), Err(abi::BAD_HANDLE));
    }
    assert_eq!(pair(2).send(0, send, b"stale"), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.handle(0, 1), Err(abi::BAD_HANDLE));
    assert!(Ipc::new(0, 3, &[]).is_err());
    assert!(Ipc::new(MAX_EPOCH + 1, 3, &[]).is_err());
}
#[test]
fn backpressure_and_small_buffer_preserve_message() {
    let mut ipc = pair(1);
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    assert_eq!(ipc.receive(1, receive, 64), Ok(None));
    assert_eq!(ipc.send(0, send, &[1; 65]), Err(abi::INVALID_ARGUMENT));
    ipc.send(0, send, b"hello").unwrap();
    assert_eq!(ipc.send(0, send, b"lost"), Err(abi::WOULD_BLOCK));
    assert_eq!(ipc.receive(1, receive, 4), Err(abi::TOO_SMALL));
    let message = ipc.receive(1, receive, 64).unwrap().unwrap();
    assert_eq!(&message.bytes[..message.len], b"hello");
    ipc.send(0, send, b"").unwrap();
    assert_eq!(ipc.receive(1, receive, 0).unwrap().unwrap().len, 0);
}
#[test]
fn revocation_cancels_pending_and_queued_messages() {
    let mut ipc = pair(1);
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    let revoke = ipc.handle(2, 0).unwrap();
    ipc.send(0, send, b"queued").unwrap();
    ipc.revoke(2, revoke).unwrap();
    assert_eq!(ipc.receive(1, receive, 64), Err(abi::REVOKED));
    assert_eq!(ipc.send(0, send, b"late"), Err(abi::REVOKED));
    assert_eq!(ipc.revoke(2, revoke), Err(abi::REVOKED));
}
#[test]
fn sender_exit_drains_accepted_message_receiver_exit_discards_it() {
    let mut ipc = pair(1);
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    ipc.send(0, send, b"last").unwrap();
    ipc.close_task(0);
    assert_eq!(ipc.handle(0, 0), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.send(0, send, b"retired"), Err(abi::BAD_HANDLE));
    assert!(ipc.receive(1, receive, 64).unwrap().is_some());
    assert_eq!(ipc.receive(1, receive, 64), Err(abi::PEER_CLOSED));
    let mut ipc = pair(1);
    ipc.send(0, send, b"discarded").unwrap();
    ipc.close_task(1);
    assert_eq!(ipc.send(0, send, b"late"), Err(abi::PEER_CLOSED));
    assert!(ipc.endpoints[0].unwrap().message.is_none());
}

#[test]
fn admission_rejects_invalid_peers_and_excessive_endpoints() {
    let endpoint = EndpointSpec {
        sender: 0,
        receiver: 1,
        revoker: None,
    };
    assert!(Ipc::new(1, 2, &[endpoint; MAX_ENDPOINTS + 1]).is_err());
    assert!(Ipc::new(1, MAX_TASKS + 1, &[]).is_err());
    assert!(Ipc::new(1, 1, &[endpoint]).is_err());
    assert!(
        Ipc::new(
            1,
            2,
            &[EndpointSpec {
                receiver: 0,
                ..endpoint
            }]
        )
        .is_err()
    );
    assert!(
        Ipc::new(
            1,
            2,
            &[EndpointSpec {
                revoker: Some(2),
                ..endpoint
            }]
        )
        .is_err()
    );
}
