use super::*;
#[test]
fn mixed_observers_do_not_consume_or_survive_slot_reuse() {
    let mut n = Notifications::new(2).unwrap();
    let [wait, signal, control] = n.create(0).unwrap();
    let peer = grant(&mut n, 0, wait, 1);
    assert_eq!(n.ready(0, signal), Err(abi::DENIED));
    n.observe(1, peer).unwrap();
    n.signal(0, signal, false).unwrap();
    assert_eq!(n.take_changed(), 2);
    assert_eq!(n.ready(1, peer), Ok(true));
    assert_eq!(n.ready(1, peer), Ok(true));
    assert_eq!(n.wait(1, peer, Some(0), 0), Ok(Some(abi::NOTIFY_SIGNALED)));
    n.unobserve(1);
    n.signal(0, signal, false).unwrap();
    assert_eq!(n.take_changed(), 0);
    n.observe(1, peer).unwrap();
    n.control(0, control, abi::NOTIFY_CLOSE).unwrap();
    assert_eq!(n.take_changed(), 2);
    assert_eq!(n.ready(1, peer), Err(abi::PEER_CLOSED));
    n.close_task(1);
    let [fresh, s, _] = n.create(0).unwrap();
    n.signal(0, s, false).unwrap();
    assert_eq!(n.take_changed(), 0);
    assert_eq!(n.ready(1, peer), Err(abi::BAD_HANDLE));
    assert_eq!(n.ready(0, fresh), Ok(true));
}
fn grant(n: &mut Notifications, from: usize, handle: u64, to: usize) -> u64 {
    let escrow = n.reserve(from, handle, to).unwrap();
    n.publish(to, escrow)
}

#[test]
fn registration_orders_and_coalescing_never_lose_an_early_signal() {
    for early in [true, false] {
        let mut n = Notifications::new(2).unwrap();
        let [w, s, _] = n.create(0).unwrap();
        let peer = grant(&mut n, 0, s, 1);
        if early {
            n.signal(1, peer, false).unwrap();
            n.signal(1, peer, true).unwrap();
        }
        assert_eq!(
            n.wait(0, w, Some(50), 0).unwrap(),
            early.then_some(abi::NOTIFY_SIGNALED)
        );
        if !early {
            n.signal(1, peer, false).unwrap();
            assert_eq!(n.complete(0, 0), Some(abi::NOTIFY_SIGNALED));
        }
        assert_eq!(n.wait(0, w, Some(10), 10), Ok(Some(abi::NOTIFY_TIMED_OUT)));
        n.signal(1, peer, false).unwrap();
        assert_eq!(n.wait(0, w, Some(10), 10), Ok(Some(abi::NOTIFY_SIGNALED)));
    }
}

#[test]
fn wake_one_all_and_completion_races_have_one_winner() {
    let mut n = Notifications::new(3).unwrap();
    let [w, s, c] = n.create(0).unwrap();
    let w1 = grant(&mut n, 0, w, 1);
    let w2 = grant(&mut n, 0, w, 2);
    n.wait(1, w1, Some(10), 0).unwrap();
    n.wait(2, w2, Some(10), 0).unwrap();
    n.signal(0, s, false).unwrap();
    assert_eq!(n.take_changed(), 2);
    n.control(0, c, abi::NOTIFY_CLOSE).unwrap();
    assert_eq!(n.complete(1, 10), Some(abi::NOTIFY_SIGNALED));
    assert_eq!(n.complete(2, 10), Some(abi::NOTIFY_CLOSED));
    assert_eq!(n.signal(0, s, false), Err(abi::PEER_CLOSED));

    let [w, s, c] = n.create(0).unwrap();
    let w1 = grant(&mut n, 0, w, 1);
    let w2 = grant(&mut n, 0, w, 2);
    n.wait(1, w1, Some(10), 0).unwrap();
    n.wait(2, w2, Some(10), 0).unwrap();
    n.signal(0, s, true).unwrap();
    n.control(0, c, abi::NOTIFY_CANCEL_WAITS).unwrap();
    assert_eq!(n.complete(1, 10), Some(abi::NOTIFY_SIGNALED));
    assert_eq!(n.complete(2, 10), Some(abi::NOTIFY_SIGNALED));
    assert_eq!(n.wait(0, w, Some(10), 10), Ok(Some(abi::NOTIFY_TIMED_OUT)));
    // Timeout commits first; a subsequent signal is retained for the next wait.
    n.wait(1, w1, Some(20), 10).unwrap();
    assert_eq!(n.complete(1, 20), Some(abi::NOTIFY_TIMED_OUT));
    n.signal(0, s, false).unwrap();
    assert_eq!(n.wait(1, w1, None, 20), Ok(Some(abi::NOTIFY_SIGNALED)));
}

#[test]
fn combined_call_distinguishes_pre_signal_errors_from_wait_outcomes() {
    let mut n = Notifications::new(2).unwrap();
    let [request, signal, _] = n.create(0).unwrap();
    let [reply, respond, close] = n.create(0).unwrap();
    assert_eq!(
        n.signal_and_wait(0, signal, respond, Some(0), 0),
        Err(abi::DENIED)
    );
    assert_eq!(
        n.wait(0, request, Some(0), 0),
        Ok(Some(abi::NOTIFY_TIMED_OUT))
    );
    assert_eq!(
        n.signal_and_wait(0, signal, reply, Some(0), 0),
        Ok(Some(abi::NOTIFY_TIMED_OUT))
    );
    assert_eq!(n.wait(0, request, None, 0), Ok(Some(abi::NOTIFY_SIGNALED)));
    // Early reply before the caller's combined operation.
    n.signal(0, respond, false).unwrap();
    assert_eq!(
        n.signal_and_wait(0, signal, reply, None, 0),
        Ok(Some(abi::NOTIFY_SIGNALED))
    );
    n.wait(0, request, None, 0).unwrap();
    let control = grant(&mut n, 0, close, 1);
    assert_eq!(n.signal_and_wait(0, signal, reply, None, 0), Ok(None));
    n.control(1, control, abi::NOTIFY_CLOSE).unwrap();
    assert_eq!(n.complete(0, 0), Some(abi::NOTIFY_CLOSED));
    assert_eq!(n.wait(0, request, None, 0), Ok(Some(abi::NOTIFY_SIGNALED)));
}

#[test]
fn cancellation_and_withdrawal_preserve_other_rights_and_completion_pins() {
    let mut n = Notifications::new(2).unwrap();
    let [w, s, c] = n.create(0).unwrap();
    let peer = grant(&mut n, 0, w, 1);
    n.wait(1, peer, None, 0).unwrap();
    n.control(0, c, abi::NOTIFY_CANCEL_WAITS).unwrap();
    assert_eq!(n.complete(1, 0), Some(abi::NOTIFY_CANCELLED));
    n.wait(1, peer, None, 0).unwrap();
    n.control(0, c, abi::NOTIFY_WITHDRAW_WAITS).unwrap();
    assert_eq!(n.complete(1, 0), Some(abi::NOTIFY_REVOKED));
    assert_eq!(n.wait(1, peer, None, 0), Err(abi::REVOKED));
    assert_eq!(n.signal(0, s, false), Ok(()));

    let [w, s, c] = n.create(0).unwrap();
    n.wait(0, w, None, 0).unwrap();
    for handle in [w, s, c] {
        n.drop_handle(0, handle).unwrap();
    }
    assert_eq!(n.charged(0), 2); // withdrawn wait still owns its completion.
    assert_eq!(n.complete(0, 0), Some(abi::NOTIFY_REVOKED));
    assert_eq!(n.charged(0), 1);
}

#[test]
fn rights_are_scoped_and_names_survive_task_and_object_reuse() {
    let mut n = Notifications::new(2).unwrap();
    let [w, s, c] = n.create(0).unwrap();
    assert_eq!(n.signal(1, s, false), Err(abi::BAD_HANDLE));
    assert_eq!(n.signal(0, w, false), Err(abi::DENIED));
    assert_eq!(n.control(0, s, 0), Err(abi::DENIED));
    let peer = grant(&mut n, 0, s, 1);
    n.close_task(0);
    assert_eq!(n.charged(0), 1);
    n.signal(1, peer, false).unwrap(); // Creator death does not close peers' object.
    let newer = n.create(0).unwrap();
    for old in [w, s, c] {
        assert_eq!(n.drop_handle(0, old), Err(abi::BAD_HANDLE));
    }
    assert!(newer.iter().all(|h| *h > c));
    n.drop_handle(1, peer).unwrap();
    assert_eq!(n.charged(0), 1);
    n.close_task(0);
    assert_eq!(n.charged(0), 0);
}

#[test]
fn checked_handoff_retains_funding_and_reclaims_abandoned_escrow() {
    use crate::ipc::{EndpointSpec, Ipc};
    let mut ipc = Ipc::with_reservations(
        1,
        2,
        &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: Some(0),
        }],
        &[1, 1],
    )
    .unwrap();
    let send = ipc.handle(0, 0).unwrap();
    let recv = ipc.handle(1, 0).unwrap();
    let [w, s, c] = ipc.create_notification(0).unwrap();
    assert_eq!(ipc.create(0), Err(abi::NO_MEMORY));
    ipc.send_notification(0, send, b"grant", s).unwrap();
    assert_eq!(ipc.receive(1, recv, 64), Err(abi::TOO_SMALL));
    assert_eq!(ipc.receive_notification(1, recv, 1), Err(abi::TOO_SMALL));
    let peer = ipc
        .receive_notification(1, recv, 64)
        .unwrap()
        .unwrap()
        .notification
        .unwrap();
    ipc.notifications.signal(1, peer, false).unwrap();
    assert_eq!(
        ipc.notifications.wait(0, w, None, 0),
        Ok(Some(abi::NOTIFY_SIGNALED))
    );
    ipc.send_notification(0, send, b"unaccepted", c).unwrap();
    assert_eq!(
        ipc.send_notification(0, send, b"full", s),
        Err(abi::WOULD_BLOCK)
    );
    assert_eq!(ipc.notifications.handles(1), 2);
    ipc.close_task(0);
    assert_eq!(ipc.charged_channels(0), 1);
    ipc.close_task(1);
    assert_eq!(ipc.charged_channels(0), 0);
    assert_eq!(ipc.notifications.handles(1), 0);
}

#[test]
fn full_recipient_table_and_withdrawal_do_not_publish_new_authority() {
    let mut n = Notifications::new(2).unwrap();
    let [wait, signal, control] = n.create(0).unwrap();
    let escrow = n.reserve(0, wait, 1).unwrap();
    let reserved_name = n.grants[escrow].unwrap().name;
    assert_eq!(n.wait(1, reserved_name, None, 0), Err(abi::BAD_HANDLE));
    n.control(0, control, abi::NOTIFY_WITHDRAW_WAITS).unwrap();
    let installed = n.publish(1, escrow);
    assert_eq!(n.wait(1, installed, None, 0), Err(abi::REVOKED));
    assert_eq!(n.reserve(0, wait, 1), Err(abi::REVOKED));
    for _ in 1..abi::NOTIFY_HANDLES {
        grant(&mut n, 0, signal, 1);
    }
    assert_eq!(n.reserve(0, signal, 1), Err(abi::NO_MEMORY));
    assert_eq!(n.handles(0), 3);
    assert_eq!(n.charged(0), 1);
    n.close_task(0);
    assert_eq!(n.charged(0), 1);
    n.close_task(1);
    assert_eq!(n.charged(0), 0);
}
