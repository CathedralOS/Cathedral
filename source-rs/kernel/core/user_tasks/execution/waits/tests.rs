use super::{Kind, Registry, slots};

#[test]
fn unrelated_events_do_not_select_waiters_and_hints_do_not_consume_a_wait() {
    let mut waits = Registry::new(31);
    waits.register(2, Kind::Receive, None, 0);
    waits.register(7, Kind::Readiness, Some(50), 0);
    waits.register(30, Kind::Child, None, 0);
    waits.watch_child(30, 4);
    assert_eq!(waits.candidates(1, 1 << 1, 1 << 2), [0; 3]);
    assert_eq!(
        waits.candidates(2, (1 << 2) | (1 << 7), 0),
        [1 << 2, 1 << 7, 0]
    );
    // A hint can concern another endpoint of the same process. Only the syscall
    // owner can decide completion, so registration survives an unsuccessful probe.
    assert_eq!(waits.candidates(3, 1 << 2, 1 << 7), [1 << 2, 1 << 7, 0]);
    waits.child_changed(5);
    assert_eq!(waits.candidates(4, 0, 0), [0; 3]);
    waits.child_changed(4);
    assert_eq!(waits.candidates(5, 0, 0), [0, 0, 1 << 30]);
    assert_eq!(waits.candidates(6, 0, 0), [0; 3]);
    assert_eq!(
        slots((1 << 30) | 1 | (1 << 4)).collect::<alloc::vec::Vec<_>>(),
        [0, 4, 30]
    );
}

#[test]
fn cancellation_clears_timer_and_child_subscription_before_slot_reuse() {
    let mut waits = Registry::new(3);
    waits.register(1, Kind::Child, Some(5), 0);
    waits.watch_child(1, 0);
    waits.child_changed(0);
    waits.remove(1);
    waits.register(1, Kind::Readiness, Some(10), 1);
    waits.child_changed(0);
    assert_eq!(waits.candidates(5, 0, 0), [0; 3]);
    assert_eq!(waits.candidates(10, 0, 0), [0, 1 << 1, 0]);
    waits.remove(1);
    assert_eq!(waits.candidates(20, u32::MAX, u32::MAX), [0; 3]);
}

#[test]
fn only_due_waiters_are_selected_across_tick_wrap_and_simultaneous_events() {
    let mut waits = Registry::new(31);
    for slot in 0..31 {
        waits.register(slot, Kind::Receive, Some(100), u64::MAX - 3);
    }
    waits.remove(0);
    waits.register(0, Kind::Readiness, Some(2), u64::MAX - 3);
    assert_eq!(waits.candidates(1, 0, 0), [0; 3]);
    assert_eq!(waits.candidates(2, 1, 1), [0, 1, 0]);
    waits.remove(0);
    assert_eq!(waits.candidates(99, 0, 0), [0; 3]);
    assert_eq!(waits.candidates(100, 0, 0), [0x7fff_fffe, 0, 0]);
    assert_eq!(waits.candidates(101, 0, 0), [0; 3]);
}

#[test]
fn populated_waiters_probe_only_the_changed_receiver_and_preserve_timeout_ties() {
    use crate::ipc::{EndpointSpec, Ipc};
    use cathedral_kernel_contracts::user as abi;
    let specs: alloc::vec::Vec<_> = (1..31)
        .map(|receiver| EndpointSpec {
            sender: 0,
            receiver,
            revoker: Some(0),
        })
        .collect();
    let mut ipc = Ipc::new(1, 31, &specs).unwrap();
    let mut waits = Registry::new(31);
    for slot in 1..31 {
        waits.register(slot, Kind::Receive, Some(100), 0);
    }
    // Each delivery selects only its receiver. Other blocked tasks need
    // no grant lookup or message retry; their deadlines remain registered.
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    ipc.send(0, send, b"first").unwrap();
    assert_eq!(
        waits.candidates(1, ipc.take_changed_receivers(), 0),
        [2, 0, 0]
    );
    assert_eq!(ipc.receive(1, receive, 0), Err(abi::TOO_SMALL));
    assert!(ipc.ready(1, receive).unwrap()); // Failed copy capacity preserves queue.
    ipc.receive(1, receive, 64).unwrap();
    ipc.send(0, send, b"second").unwrap();
    assert_eq!(
        waits.candidates(2, ipc.take_changed_receivers(), 0),
        [2, 0, 0]
    );
    // At equality the waiter is selected once, and its owner can still observe
    // queued data before testing expiry. Dead sender is also a terminal result.
    assert_eq!(waits.candidates(100, 2, 0), [0x7fff_fffe, 0, 0]);
    assert_eq!(ipc.receive(1, receive, 64).unwrap().unwrap().len, 6);
    ipc.close_task(0);
    assert_eq!(ipc.receive(1, receive, 64), Err(abi::PEER_CLOSED));
}
