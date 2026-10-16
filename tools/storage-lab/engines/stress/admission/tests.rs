use super::*;
use budget::OBJECT_CHARGE;
use std::time::Duration;

fn request(budget: &Budget, client: Client, bytes: usize) -> Request {
    let reservation = budget.reserve(client, &[bytes]).unwrap();
    let now = Instant::now();
    Request::prepared(
        reservation,
        vec![Prepared::new(1, 0, vec![1; bytes])],
        now,
        now,
    )
    .unwrap()
}
fn empty(budget: &Budget) {
    let state = budget.snapshot();
    assert_eq!(state.current.bytes, 0);
    assert_eq!(state.current.requests, 0);
    assert!(
        state
            .clients
            .iter()
            .all(|c| c.bytes == 0 && c.requests == 0)
    );
}

#[test]
fn transaction_boundaries_include_record_overhead_and_reject_overflow() {
    let limits = Limits::default();
    let budget = Budget::new(limits);
    drop(
        budget
            .reserve(Client::Bulk, &[limits.transaction_bytes - OBJECT_CHARGE])
            .unwrap(),
    );
    for lengths in [
        vec![],
        vec![limits.transaction_bytes - OBJECT_CHARGE + 1],
        vec![0; 9],
        vec![usize::MAX],
        vec![usize::MAX - OBJECT_CHARGE, 1],
    ] {
        assert_eq!(
            budget.reserve(Client::Bulk, &lengths).unwrap_err(),
            Denied::TooLarge
        );
        empty(&budget);
    }
}

#[test]
fn independent_client_caps_leave_room_for_other_producer() {
    let budget = Budget::new(Limits::default());
    let bulk: Vec<_> = (0..3)
        .map(|_| budget.reserve(Client::Bulk, &[64 * 1024]).unwrap())
        .collect();
    assert_eq!(
        budget.reserve(Client::Bulk, &[64 * 1024]).unwrap_err(),
        Denied::Busy
    );
    let small = budget.reserve(Client::Interactive, &[4096]).unwrap();
    drop(bulk);
    assert_eq!(budget.snapshot().current.bytes, 4096 + OBJECT_CHARGE);
    drop(small);
    empty(&budget);
}

#[test]
fn global_bytes_and_request_slots_are_independently_enforced() {
    let budget = Budget::new(Limits {
        bytes: 1024,
        client_bytes: 1024,
        ..Limits::default()
    });
    let first = budget.reserve(Client::Bulk, &[768]).unwrap();
    assert_eq!(
        budget.reserve(Client::Interactive, &[0]).unwrap_err(),
        Denied::Busy
    );
    assert_eq!(
        budget
            .wait(Client::Interactive, &[0], Duration::ZERO)
            .unwrap_err(),
        Denied::Timeout
    );
    drop(first);
    empty(&budget);
    let budget = Budget::new(Limits {
        requests: 2,
        client_requests: 1,
        ..Limits::default()
    });
    let bulk = budget.reserve(Client::Bulk, &[0]).unwrap();
    assert_eq!(
        budget.reserve(Client::Bulk, &[0]).unwrap_err(),
        Denied::Busy
    );
    let small = budget.reserve(Client::Interactive, &[0]).unwrap();
    assert_eq!(
        budget.reserve(Client::Interactive, &[0]).unwrap_err(),
        Denied::Busy
    );
    drop((bulk, small));
    empty(&budget);
}

#[test]
fn charge_survives_prepare_queue_and_active_writer_until_drop() {
    let budget = Budget::new(Limits::default());
    let queue = Queue::new(budget.clone());
    let request = request(&budget, Client::Bulk, 1024);
    let charge = budget.snapshot().current.bytes;
    queue.push(request).unwrap();
    assert_eq!(budget.snapshot().current.bytes, charge);
    let active = queue.pop(Duration::ZERO).unwrap();
    assert_eq!(budget.snapshot().current.bytes, charge);
    queue.close();
    assert_eq!(budget.snapshot().current.bytes, charge);
    drop(active);
    empty(&budget);
}

#[test]
fn ready_clients_alternate_fifo_and_preparing_client_cannot_block_dispatch() {
    let budget = Budget::new(Limits::default());
    let queue = Queue::new(budget.clone());
    let base = Instant::now();
    for i in 0..4 {
        let mut item = request(&budget, Client::Bulk, 1);
        item.started = base + Duration::from_millis(i);
        queue.push(item).unwrap();
    }
    for i in 0..4 {
        let mut item = request(&budget, Client::Interactive, 1);
        item.started = base + Duration::from_millis(i);
        queue.push(item).unwrap();
    }
    for i in 0..4 {
        for client in [Client::Interactive, Client::Bulk] {
            let item = queue.pop(Duration::ZERO).unwrap();
            assert_eq!(item.reservation.client(), client);
            assert_eq!(item.started, base + Duration::from_millis(i));
        }
    }
    let preparing = budget.reserve(Client::Interactive, &[1024]).unwrap();
    queue.push(request(&budget, Client::Bulk, 1)).unwrap();
    assert_eq!(
        queue.pop(Duration::ZERO).unwrap().reservation.client(),
        Client::Bulk
    );
    drop(preparing);
    empty(&budget);
}

#[test]
fn wrong_payload_and_foreign_budget_cannot_enter_queue() {
    let budget = Budget::new(Limits::default());
    let reservation = budget.reserve(Client::Bulk, &[1]).unwrap();
    let now = Instant::now();
    assert!(matches!(
        Request::prepared(reservation, vec![Prepared::new(1, 0, vec![1; 2])], now, now),
        Err(Denied::Mismatch)
    ));
    empty(&budget);
    let other = Budget::new(Limits::default());
    let queue = Queue::new(budget.clone());
    assert_eq!(
        queue.push(request(&other, Client::Bulk, 1)),
        Err(Denied::Mismatch)
    );
    empty(&other);
    empty(&budget);
}

#[test]
fn cancellation_close_and_preparation_panic_return_reservations() {
    let budget = Budget::new(Limits::default());
    assert!(
        std::panic::catch_unwind(|| {
            let _reservation = budget.reserve(Client::Bulk, &[1024]).unwrap();
            panic!("preparation failed");
        })
        .is_err()
    );
    empty(&budget);
    let queue = Queue::new(budget.clone());
    let pending = request(&budget, Client::Bulk, 1);
    queue
        .push(request(&budget, Client::Interactive, 1))
        .unwrap();
    budget.close();
    queue.close();
    assert_eq!(queue.push(pending), Err(Denied::Closed));
    assert!(matches!(queue.pop(Duration::ZERO), Err(Denied::Closed)));
    assert_eq!(
        budget.wait(Client::Bulk, &[1], Duration::ZERO).unwrap_err(),
        Denied::Closed
    );
    empty(&budget);
}

#[test]
fn conflict_and_engine_io_failure_do_not_leak_inflight_charge() {
    use super::super::device::{Cut, Device};
    let device = Device::new(Vec::new());
    let db = publication::open(device.clone()).unwrap();
    publication::initialize(&db).unwrap();
    let budget = Budget::new(Limits::default());
    let queue = Queue::new(budget.clone());
    let first = request(&budget, Client::Interactive, 1024);
    assert!(
        publication::publish(&db, &first.changes, false)
            .unwrap()
            .accepted
    );
    drop(first);
    let stale = request(&budget, Client::Interactive, 1024);
    assert!(
        !publication::publish(&db, &stale.changes, false)
            .unwrap()
            .accepted
    );
    drop(stale);
    empty(&budget);
    let reservation = budget.reserve(Client::Bulk, &[4096]).unwrap();
    let now = Instant::now();
    let active = Request::prepared(
        reservation,
        vec![Prepared::new(2, 0, vec![2; 4096])],
        now,
        now,
    )
    .unwrap();
    queue
        .push(request(&budget, Client::Interactive, 1024))
        .unwrap();
    device.arm(1, Cut::Before);
    assert!(publication::publish(&db, &active.changes, false).is_err());
    queue.close();
    assert_eq!(budget.snapshot().current.bytes, 4096 + OBJECT_CHARGE);
    drop(active);
    empty(&budget);
}
