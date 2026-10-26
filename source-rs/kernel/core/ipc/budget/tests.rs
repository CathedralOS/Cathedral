use super::*;
#[test]
fn high_account_liveness_and_reclamation_do_not_alias_low_accounts() {
    let mut limits = [0; abi::MAX_TASKS];
    let last = abi::MAX_TASKS - 1;
    limits[last] = 1;
    let mut ipc = Ipc::with_reservations(1, abi::MAX_TASKS, &[], &limits).unwrap();
    let [send, receive] = ipc.create(last).unwrap();
    ipc.send(last, send, b"high account").unwrap();
    ipc.close_task(last - 8);
    assert_eq!(ipc.receive(last, receive, 64).unwrap().unwrap().len, 12);
    ipc.close_task(last);
    assert_eq!(ipc.charged_channels(last), 0);
    assert!(ipc.create(last).is_err());
}
#[test]
fn reservation_validation_rejects_overcommit_and_keeps_defaults_constant() {
    assert_eq!(&reservations(8, &[]).unwrap()[..8], &[4; 8]);
    assert_eq!(&reservations(2, &[]).unwrap()[..2], &[4; 2]);
    for (tasks, limits) in [
        (
            abi::MAX_DYNAMIC_CHANNELS / abi::DEFAULT_CHANNELS + 1,
            &[][..],
        ),
        (2, &[1][..]),
        (1, &[abi::MAX_CREATED_CHANNELS + 1][..]),
        (8, &[abi::MAX_DYNAMIC_CHANNELS / 8 + 1; 8][..]),
    ] {
        assert_eq!(reservations(tasks, limits), Err(abi::INVALID_ARGUMENT));
        assert!(Ipc::with_reservations(1, tasks, &[], limits).is_err());
    }
    let mut ipc = Ipc::with_reservations(1, 2, &[], &[0, 8]).unwrap();
    assert_eq!(ipc.create(0), Err(abi::NO_MEMORY));
    assert_eq!(
        ipc.budget(0),
        Ok([0, 0, abi::MAX_DYNAMIC_HANDLES as u64, 0])
    );
    for _ in 0..8 {
        ipc.create(1).unwrap();
    }
}
#[test]
fn every_task_can_fill_its_reservation_even_after_other_accounts_exhaust() {
    // Keep endpoint handles spare while filling the entire global object arena.
    let tasks = 16;
    let per_task = abi::MAX_DYNAMIC_CHANNELS / tasks;
    let mut ipc =
        Ipc::with_reservations(1, tasks, &[], &[abi::MAX_DYNAMIC_CHANNELS / 16; 16]).unwrap();
    let mut all = alloc::vec::Vec::new();
    for task in 0..tasks {
        for _ in 0..per_task {
            all.push((task, ipc.create(task).unwrap()));
        }
        let before = ipc.budget(task).unwrap();
        assert_eq!(ipc.create(task), Err(abi::NO_MEMORY));
        assert_eq!(ipc.budget(task), Ok(before));
    }
    assert_eq!(
        ipc.endpoints[MAX_ENDPOINTS..].iter().flatten().count(),
        abi::MAX_DYNAMIC_CHANNELS
    );
    let (task, [s, r]) = all[0];
    ipc.close(task, s).unwrap();
    assert_eq!(ipc.create(task), Err(abi::NO_MEMORY)); // One surviving end still pins storage.
    ipc.close(task, r).unwrap();
    assert_eq!(ipc.close(task, s), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.create(1), Err(abi::NO_MEMORY)); // Free capacity is still task 0's reservation.
    let fresh = ipc.create(task).unwrap();
    assert_ne!(fresh[0], s);
    assert_eq!(ipc.send(task, s, b"stale"), Err(abi::BAD_HANDLE));
    for task in 0..tasks {
        ipc.close_task(task);
    }
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
}
#[test]
fn restart_cannot_erase_charges_for_transferred_channels_and_sibling_progresses() {
    let mut ipc = Ipc::with_reservations(1, 3, &[], &[2, 2, 2]).unwrap();
    ipc.close_task(1);
    let mut held = alloc::vec::Vec::new();
    for round in 0..2 {
        ipc.prepare_child(0, 0, 1, 2 + round);
        ipc.accept_child(0, 0);
        assert_eq!(
            ipc.budget(1),
            Ok([2, round, abi::MAX_DYNAMIC_HANDLES as u64, 0])
        );
        let [s, r] = ipc.create(1).unwrap();
        ipc.send_cap(1, ipc.handle(1, 1).unwrap(), b"hold", Some(r))
            .unwrap();
        let received = ipc
            .receive_cap(0, ipc.handle(0, 1).unwrap(), 64, true)
            .unwrap()
            .unwrap()
            .attachment
            .unwrap();
        held.push(received);
        ipc.send(1, s, b"queued before death").unwrap();
        ipc.close_task(1);
        assert_eq!(ipc.budget(1), Err(abi::BAD_HANDLE));
    }
    ipc.prepare_child(0, 0, 1, 4);
    ipc.accept_child(0, 0);
    assert_eq!(
        ipc.budget(1),
        Ok([2, 2, abi::MAX_DYNAMIC_HANDLES as u64, 0])
    );
    assert_eq!(ipc.create(1), Err(abi::NO_MEMORY));
    let before = ipc.budget(0).unwrap();
    for _ in 0..2 {
        ipc.create(2).unwrap();
    }
    assert_eq!(ipc.budget(0), Ok(before));
    assert_eq!(ipc.receive(0, held[0], 64).unwrap().unwrap().len, 19);
    assert_eq!(ipc.receive(0, held[0], 64), Err(abi::PEER_CLOSED));
    ipc.close(0, held[0]).unwrap();
    assert_eq!(
        ipc.budget(1),
        Ok([2, 1, abi::MAX_DYNAMIC_HANDLES as u64, 0])
    );
    ipc.create(1).unwrap();
    for task in 0..3 {
        ipc.close_task(task);
    }
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
}
#[test]
fn escrow_charges_custody_separately_and_refunds_on_discard() {
    for receiver_dies in [false, true] {
        let mut ipc = Ipc::new(
            1,
            3,
            &[EndpointSpec {
                sender: 0,
                receiver: 1,
                revoker: Some(2),
            }],
        )
        .unwrap();
        let [s, r] = ipc.create(0).unwrap();
        let before = ipc.budget(0).unwrap();
        assert_eq!(
            ipc.send_cap(0, ipc.handle(0, 0).unwrap(), b"bad", Some(0)),
            Err(abi::BAD_HANDLE)
        );
        assert_eq!(ipc.budget(0), Ok(before));
        ipc.send_cap(0, ipc.handle(0, 0).unwrap(), b"escrow", Some(r))
            .unwrap();
        assert_eq!(
            ipc.budget(0),
            Ok([4, 1, abi::MAX_DYNAMIC_HANDLES as u64, 1])
        );
        assert_eq!(
            ipc.budget(1),
            Ok([4, 0, abi::MAX_DYNAMIC_HANDLES as u64, 1])
        ); // Invisible incoming grant already consumes capacity.
        ipc.close(0, s).unwrap();
        ipc.close_task(0);
        assert_eq!(ipc.charged_channels(0), 1);
        if receiver_dies {
            ipc.close_task(1);
        } else {
            ipc.revoke(2, ipc.handle(2, 0).unwrap()).unwrap();
            assert_eq!(
                ipc.budget(1),
                Ok([4, 0, abi::MAX_DYNAMIC_HANDLES as u64, 0])
            );
        }
        assert_eq!(ipc.charged_channels(0), 0);
    }
}
