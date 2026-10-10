use super::*;
#[test]
fn graph_replacement_changes_only_edges_touching_the_replaced_task() {
    let mut ipc = Ipc::new(1, 4, &[]).unwrap();
    for child in 1..4 {
        ipc.close_task(child);
    }
    for child in 1..4 {
        ipc.prepare_child((child - 1) * 2, 0, child, child as u64 + 1);
        ipc.accept_child((child - 1) * 2, 0);
    }
    for (base, service, epoch) in [(6, 1, 5), (8, 2, 6)] {
        ipc.prepare_pair(base, 3, service, epoch);
        ipc.accept_child(base, 3);
        ipc.accept_child(base, service);
    }
    let controls: [_; 6] = core::array::from_fn(|i| ipc.handle(0, i as u64).unwrap());
    let app_control = ipc.handle(3, 0).unwrap();
    let old = ipc.handle(3, 2).unwrap();
    let stable = ipc.handle(3, 4).unwrap();
    ipc.send(3, old, b"discard").unwrap();
    ipc.send(3, stable, b"preserve").unwrap();
    ipc.close_task(1);
    ipc.prepare_child(0, 0, 1, 7);
    ipc.accept_child(0, 0);
    ipc.prepare_pair(6, 3, 1, 8);
    assert_eq!(ipc.send(3, old, b"stale"), Err(abi::BAD_HANDLE));
    ipc.accept_child(6, 3);
    ipc.accept_child(6, 1);
    assert_eq!(ipc.handle(3, 0), Ok(app_control));
    assert_eq!(ipc.handle(3, 4), Ok(stable));
    let input = ipc.handle(2, 2).unwrap();
    assert_eq!(ipc.receive(2, input, 64).unwrap().unwrap().len, 8);
    let input = ipc.handle(1, 2).unwrap();
    assert_eq!(ipc.receive(1, input, 64), Ok(None));
    // Replacing the application refreshes both edges, preserving provider controls.
    let old_data = [ipc.handle(1, 2).unwrap(), ipc.handle(2, 2).unwrap()];
    ipc.close_task(3);
    ipc.prepare_child(4, 0, 3, 9);
    ipc.accept_child(4, 0);
    for (base, service, epoch) in [(6, 1, 10), (8, 2, 11)] {
        ipc.prepare_pair(base, 3, service, epoch);
        assert_eq!(
            ipc.ready(service, old_data[service - 1]),
            Err(abi::BAD_HANDLE)
        );
        ipc.accept_child(base, service);
        ipc.accept_child(base, 3);
    }
    assert_eq!(ipc.handle(0, 2), Ok(controls[2]));
    assert_eq!(ipc.handle(0, 3), Ok(controls[3]));
    assert_ne!(ipc.handle(0, 4), Ok(controls[4]));
}
#[test]
fn readiness_preserves_messages_checks_rights_and_exposes_closure() {
    let mut ipc = pair(1);
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    assert_eq!(ipc.ready(0, send), Err(abi::DENIED));
    assert_eq!(ipc.ready(1, send), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.ready(1, receive), Ok(false));
    ipc.send(0, send, b"queued").unwrap();
    assert_eq!(ipc.ready(1, receive), Ok(true));
    assert_eq!(ipc.ready(1, receive), Ok(true));
    assert_eq!(ipc.receive(1, receive, 64).unwrap().unwrap().len, 6);
    assert_eq!(ipc.ready(1, receive), Ok(false));
    ipc.close_task(0);
    assert_eq!(ipc.ready(1, receive), Ok(true));
    assert_eq!(ipc.receive(1, receive, 64), Err(abi::PEER_CLOSED));
    let revoke = ipc.handle(2, 0).unwrap();
    ipc.revoke(2, revoke).unwrap();
    assert_eq!(ipc.ready(1, receive), Err(abi::REVOKED));
}
#[test]
fn restarting_either_sibling_preserves_other_tickets_and_queued_reply() {
    let mut ipc = Ipc::new(1, 3, &[]).unwrap();
    ipc.close_task(1);
    ipc.close_task(2);
    ipc.prepare_child(0, 0, 1, 2);
    ipc.prepare_child(2, 0, 2, 3);
    assert_eq!(ipc.accept_child(2, 0), 0); // earlier pair still hidden
    assert_eq!(ipc.accept_child(0, 0), 0);
    for (child, base, sibling, sibling_base, epoch) in [(1, 0, 2, 2, 4), (2, 2, 1, 0, 5)] {
        let old = ipc.handle(0, base as u64).unwrap();
        let stable = ipc.handle(0, sibling_base as u64).unwrap();
        let reply = ipc.handle(0, sibling_base as u64 + 1).unwrap();
        let output = ipc.handle(sibling, 1).unwrap();
        ipc.send(sibling, output, b"still queued").unwrap();
        ipc.close_task(child);
        ipc.prepare_child(base, 0, child, epoch);
        assert_eq!(ipc.accept_child(base, 0), base as u64);
        assert_eq!(ipc.send(0, old, b"stale"), Err(abi::BAD_HANDLE));
        assert_eq!(ipc.handle(0, sibling_base as u64), Ok(stable));
        let message = ipc.receive(0, reply, 64).unwrap().unwrap();
        assert_eq!(&message.bytes[..message.len], b"still queued");
        ipc.send(0, stable, b"sibling alive").unwrap();
        let input = ipc.handle(sibling, 0).unwrap();
        assert_eq!(ipc.receive(sibling, input, 64).unwrap().unwrap().len, 13);
    }
}
#[test]
fn replacing_child_grants_requires_consent_and_preserves_control_channels() {
    let mut ipc = Ipc::new(
        1,
        3,
        &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: None,
        }],
    )
    .unwrap();
    let control = ipc.handle(0, 0).unwrap();
    ipc.close_task(2);
    ipc.send(0, control, b"control").unwrap();
    ipc.prepare_child(1, 1, 2, 2);
    let old_child = ipc.handle(2, 0).unwrap();
    let guessed_peer = Ipc::ticket(2, 1, 3);
    assert_eq!(ipc.handle(1, 1), Err(abi::BAD_HANDLE));
    assert_eq!(
        ipc.send(1, guessed_peer, b"no consent"),
        Err(abi::BAD_HANDLE)
    );
    ipc.accept_child(1, 1);
    let old_peer = ipc.handle(1, 1).unwrap();
    ipc.send(1, old_peer, b"old queue").unwrap();
    ipc.close_task(2);
    assert_eq!(ipc.send(1, old_peer, b"dead"), Err(abi::PEER_CLOSED));
    ipc.prepare_child(1, 1, 2, 3);
    assert_eq!(ipc.receive(2, old_child, 64), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.send(1, old_peer, b"stale"), Err(abi::BAD_HANDLE));
    ipc.accept_child(1, 1);
    let fresh = ipc.handle(1, 1).unwrap();
    assert_ne!(old_peer, fresh);
    let input = ipc.handle(2, 0).unwrap();
    assert_eq!(ipc.receive(2, input, 64), Ok(None));
    ipc.send(1, fresh, b"new queue").unwrap();
    assert_eq!(ipc.receive(2, input, 64).unwrap().unwrap().len, 9);
    assert_eq!(ipc.handle(0, 0), Ok(control));
    let control_input = ipc.handle(1, 0).unwrap();
    assert_eq!(ipc.receive(1, control_input, 64).unwrap().unwrap().len, 7);
}
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
