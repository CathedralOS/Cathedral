use super::*;
fn setup() -> (Ipc, u64, u64) {
    let ipc = pair(1);
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    (ipc, send, receive)
}
fn accept(ipc: &mut Ipc, caller: usize, receive: u64) -> u64 {
    ipc.receive_cap(caller, receive, 64, true)
        .unwrap()
        .unwrap()
        .attachment
        .unwrap()
}
fn empty(ipc: &Ipc) {
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
    assert!(
        ipc.grants
            .entries
            .iter()
            .flatten()
            .all(|g| g.endpoint < MAX_ENDPOINTS)
    );
}
#[test]
fn ordinary_worker_gets_a_real_endpoint_not_authority_from_bytes() {
    let (mut ipc, bootstrap, receive) = setup();
    let [reply, answer] = ipc.create(0).unwrap();
    assert_eq!(ipc.send(1, reply, b"forged"), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.handle(0, 1), Err(abi::BAD_HANDLE));
    ipc.send_cap(0, bootstrap, b"work", Some(reply)).unwrap();
    assert_eq!(ipc.send(0, reply, b"moved"), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.send(1, 1, b"escrow"), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.receive(1, receive, 64), Err(abi::TOO_SMALL));
    assert_eq!(ipc.receive_cap(1, receive, 3, true), Err(abi::TOO_SMALL));
    let granted = accept(&mut ipc, 1, receive);
    assert_eq!(ipc.receive(1, granted, 64), Err(abi::DENIED));
    ipc.send(1, granted, b"done").unwrap();
    ipc.close_task(1);
    assert_eq!(ipc.receive(0, answer, 64).unwrap().unwrap().len, 4);
    assert_eq!(ipc.receive(0, answer, 64), Err(abi::PEER_CLOSED));
    ipc.close(0, answer).unwrap();
    empty(&ipc);
}
#[test]
fn failed_transfer_keeps_source_and_queue_untouched() {
    let (mut ipc, bootstrap, receive) = setup();
    let [send, recv] = ipc.create(0).unwrap();
    assert_eq!(
        ipc.send_cap(0, bootstrap, b"bad", Some(bootstrap)),
        Err(abi::DENIED)
    );
    assert_eq!(
        ipc.send_cap(0, bootstrap, b"bad", Some(0)),
        Err(abi::BAD_HANDLE)
    );
    assert_eq!(
        ipc.send_cap(0, bootstrap, &[0; 65], Some(send)),
        Err(abi::INVALID_ARGUMENT)
    );
    ipc.send(0, bootstrap, b"first").unwrap();
    assert_eq!(
        ipc.send_cap(0, bootstrap, b"second", Some(send)),
        Err(abi::WOULD_BLOCK)
    );
    ipc.send(0, send, b"still owned").unwrap();
    assert_eq!(ipc.receive(0, recv, 64).unwrap().unwrap().len, 11);
    assert_eq!(ipc.receive(1, receive, 64).unwrap().unwrap().len, 5);
    ipc.close_task(1);
    assert_eq!(
        ipc.send_cap(0, bootstrap, b"dead", Some(send)),
        Err(abi::PEER_CLOSED)
    );
    ipc.check(0, send, Right::Send).unwrap();
}
#[test]
fn receiver_budget_is_reserved_before_move_and_released_on_discard() {
    let (mut ipc, bootstrap, receive) = setup();
    let full: alloc::vec::Vec<_> = (0..abi::MAX_CREATED_CHANNELS)
        .map(|_| ipc.create(1).unwrap())
        .collect();
    let [send, recv] = ipc.create(0).unwrap();
    assert_eq!(
        ipc.send_cap(0, bootstrap, b"full", Some(send)),
        Err(abi::NO_MEMORY)
    );
    assert_eq!(ipc.receive(1, receive, 64), Ok(None));
    ipc.check(0, send, Right::Send).unwrap();
    ipc.close(1, full[0][0]).unwrap();
    ipc.send_cap(0, bootstrap, b"reserved", Some(send)).unwrap();
    assert_eq!(ipc.grants.vacant(1, 1), Err(abi::NO_MEMORY));
    let revoke = ipc.handle(2, 0).unwrap();
    ipc.revoke(2, revoke).unwrap();
    assert_eq!(ipc.receive(0, recv, 64), Err(abi::PEER_CLOSED));
    assert!(ipc.grants.vacant(1, 1).is_ok());
    for t in 0..3 {
        ipc.close_task(t);
    }
    empty(&ipc);
}
#[test]
fn queued_move_survives_sender_exit_but_not_receiver_exit() {
    for recipient_dies in [false, true] {
        let (mut ipc, bootstrap, receive) = setup();
        let [send, recv] = ipc.create(0).unwrap();
        ipc.send_cap(0, bootstrap, b"move", Some(recv)).unwrap();
        ipc.send(0, send, b"last bytes").unwrap();
        if recipient_dies {
            ipc.close_task(1);
            assert_eq!(ipc.send(0, send, b"late"), Err(abi::PEER_CLOSED));
            ipc.close_task(0);
        } else {
            ipc.close_task(0);
            let delivered = accept(&mut ipc, 1, receive);
            assert_eq!(ipc.receive(1, delivered, 64).unwrap().unwrap().len, 10);
            assert_eq!(ipc.receive(1, delivered, 64), Err(abi::PEER_CLOSED));
            ipc.close_task(1);
        }
        empty(&ipc);
    }
}
#[test]
fn quota_pool_and_local_name_exhaustion_are_atomic_and_reclaimable() {
    let mut ipc = Ipc::new(1, 8, &[]).unwrap();
    let mut all = alloc::vec::Vec::new();
    for task in 0..4 {
        for _ in 0..abi::MAX_CREATED_CHANNELS {
            all.push((task, ipc.create(task).unwrap()));
        }
        assert_eq!(ipc.create(task), Err(abi::NO_MEMORY));
    }
    assert_eq!(ipc.create(4), Err(abi::NO_MEMORY));
    let (task, [s, r]) = all.remove(0);
    ipc.close(task, s).unwrap();
    ipc.close(task, r).unwrap();
    assert_eq!(ipc.close(task, s), Err(abi::BAD_HANDLE));
    let fresh = ipc.create(4).unwrap();
    ipc.close(4, fresh[0]).unwrap();
    ipc.close(4, fresh[1]).unwrap();
    let fresh = ipc.create(task).unwrap();
    assert_ne!(fresh[0], s);
    assert_eq!(ipc.send(task, s, b"stale"), Err(abi::BAD_HANDLE));
    for task in 0..8 {
        ipc.close_task(task);
    }
    empty(&ipc);
}
#[test]
fn receive_moves_preserve_rights_and_refuse_queued_or_self_containment() {
    let (mut ipc, bootstrap, receive) = setup();
    let [s, r] = ipc.create(0).unwrap();
    assert_eq!(ipc.send_cap(0, s, b"cycle", Some(r)), Err(abi::DENIED));
    ipc.send(0, s, b"pending").unwrap();
    assert_eq!(ipc.send_cap(0, bootstrap, b"busy", Some(r)), Err(abi::BUSY));
    ipc.receive(0, r, 64).unwrap();
    ipc.send_cap(0, bootstrap, b"receive", Some(r)).unwrap();
    let r = accept(&mut ipc, 1, receive);
    ipc.send(0, s, b"data").unwrap();
    assert_eq!(ipc.receive(1, r, 64).unwrap().unwrap().len, 4);
    ipc.close(1, r).unwrap();
    assert_eq!(ipc.send(0, s, b"closed"), Err(abi::PEER_CLOSED));
    ipc.close(0, s).unwrap();
    empty(&ipc);
}
#[test]
fn chains_of_escrow_are_discarded_without_recursive_kernel_stack_growth() {
    for order in [[0, 1, 2], [1, 0, 2], [2, 1, 0]] {
        let (mut ipc, bootstrap, _) = setup();
        let mut carrier = bootstrap;
        for _ in 0..abi::MAX_CREATED_CHANNELS {
            let [s, r] = ipc.create(0).unwrap();
            ipc.send_cap(0, carrier, b"nested escrow", Some(r)).unwrap();
            carrier = s;
        }
        for task in order {
            ipc.close_task(task);
        }
        empty(&ipc);
    }
}
#[test]
fn queued_receiver_cannot_be_moved_into_its_own_descendant() {
    let (mut ipc, _, _) = setup();
    let [a, ar] = ipc.create(0).unwrap();
    let [b, br] = ipc.create(0).unwrap();
    ipc.send_cap(0, b, b"first", Some(ar)).unwrap();
    assert_eq!(ipc.send_cap(0, a, b"cycle", Some(br)), Err(abi::BUSY));
    for task in 0..3 {
        ipc.close_task(task);
    }
    empty(&ipc);
}
#[test]
fn recipient_can_forward_the_grant_to_an_unrelated_peer() {
    let mut ipc = Ipc::new(
        1,
        3,
        &[
            EndpointSpec {
                sender: 0,
                receiver: 1,
                revoker: None,
            },
            EndpointSpec {
                sender: 1,
                receiver: 2,
                revoker: None,
            },
        ],
    )
    .unwrap();
    let [s, r] = ipc.create(0).unwrap();
    ipc.send_cap(0, ipc.handle(0, 0).unwrap(), b"hop", Some(s))
        .unwrap();
    let s = accept(&mut ipc, 1, 2);
    ipc.send_cap(1, ipc.handle(1, 1).unwrap(), b"hop", Some(s))
        .unwrap();
    let s = accept(&mut ipc, 2, 2);
    ipc.close_task(1);
    ipc.send(2, s, b"peer reply").unwrap();
    assert_eq!(ipc.receive(0, r, 64).unwrap().unwrap().len, 10);
    for task in [2, 0] {
        ipc.close_task(task);
    }
    empty(&ipc);
}
#[test]
fn peer_replacement_discards_escrow_without_rebinding_transferred_channels() {
    let mut ipc = Ipc::new(1, 2, &[]).unwrap();
    ipc.close_task(1);
    ipc.prepare_child(0, 0, 1, 2);
    ipc.accept_child(0, 0);
    let bootstrap = ipc.handle(0, 0).unwrap();
    let [s, r] = ipc.create(0).unwrap();
    ipc.send_cap(0, bootstrap, b"pending", Some(s)).unwrap();
    ipc.close_task(1);
    ipc.prepare_child(0, 0, 1, 3);
    ipc.accept_child(0, 0);
    assert_eq!(ipc.receive(0, r, 64), Err(abi::PEER_CLOSED));
    assert_eq!(
        ipc.receive_cap(1, ipc.handle(1, 0).unwrap(), 64, true),
        Ok(None)
    );
    ipc.close(0, r).unwrap();
    empty(&ipc);
}
