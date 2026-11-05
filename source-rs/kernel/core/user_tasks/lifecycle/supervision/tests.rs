use super::*;

#[test]
fn asynchronous_admission_retains_one_owner_scoped_result_until_consumed() {
    let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
    model.request_async(0, 1, 99, 10).unwrap();
    assert_eq!(model.pending(), Some(99));
    assert_eq!(model.poll_admission(1, 10, true), Err(abi::BAD_HANDLE));
    assert_eq!(model.poll_admission(0, 11, true), Err(abi::BAD_HANDLE));
    assert_eq!(model.admission_ready(), Ok(()));
    assert_eq!(model.poll_admission(0, 10, false), Ok(None));
    assert_eq!(model.request(0, 1, 0), Err(abi::BUSY));
    model.failed();
    assert!(model.retain_admission(Err(abi::BAD_EXECUTABLE)));
    assert!(!model.idle());
    assert_eq!(model.can_lend(0, 1, 1), Err(abi::BUSY));
    // Failed copyout can retry without losing or changing the outcome.
    for _ in 0..2 {
        assert_eq!(
            model.poll_admission(0, 10, false),
            Ok(Some(Err(abi::BAD_EXECUTABLE)))
        );
    }
    model.consume_admission();
    assert!(model.idle());
    model.request_async(0, 1, 100, 12).unwrap();
    assert_eq!(model.poll_admission(0, 10, false), Err(abi::BAD_HANDLE));
    assert_eq!(model.pending(), Some(100));
}

#[test]
fn pending_admission_cancellation_prevents_commit_but_waits_for_rollback() {
    let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
    model.request_async(0, 1, 99, 10).unwrap();
    assert_eq!(model.poll_admission(0, 10, true), Ok(None));
    assert_eq!(model.admission_ready(), Err(abi::REVOKED));
    assert_eq!(model.poll_admission(0, 10, false), Ok(None));
    assert!(!model.idle());
    // Only the boot-stack worker, after releasing all draft backing, finishes.
    model.failed();
    assert!(model.retain_admission(Err(abi::REVOKED)));
    assert_eq!(
        model.poll_admission(0, 10, false),
        Ok(Some(Err(abi::REVOKED)))
    );
    model.consume_admission();
    assert!(model.idle());
}

#[test]
fn committed_admission_wins_a_late_cancellation_and_keeps_child_authority() {
    let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
    model.request_async(0, 1, 99, 10).unwrap();
    let child = model.started(20);
    assert!(model.retain_admission(Ok(child)));
    assert_eq!(model.poll_admission(0, 10, true), Ok(Some(Ok(child))));
    assert!(!model.cancel_child());
    model.consume_admission();
    assert_eq!(model.wait(0, child), Ok(None));
    assert!(model.request_cancel(0, child).unwrap());
    assert!(model.cancel_child());
}

#[test]
fn owner_death_withdraws_admission_receipts_and_never_revives_them_for_a_lender() {
    for completed in [false, true] {
        let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
        model.lend(1, [20, 21], [30, 31]);
        model.request_async(1, 20, 99, 40).unwrap();
        if completed {
            model.failed();
            assert!(model.retain_admission(Err(abi::NO_MEMORY)));
        }
        model.close(1);
        assert_eq!(model.pending(), None);
        assert!(!model.has_receipt(1, 40));
        model.reclaim();
        assert_eq!(model.launch(0), Ok(30));
        assert!(model.idle());
        assert_eq!(model.poll_admission(0, 40, false), Err(abi::BAD_HANDLE));
    }
}

#[test]
fn blocking_admission_still_completes_the_waiting_syscall() {
    let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
    model.request(0, 1, 99).unwrap();
    let child = model.started(20);
    assert!(!model.retain_admission(Ok(child)));
    assert_eq!(model.wait(0, child), Ok(None));
}

#[test]
fn queued_admission_rechecks_peer_death_before_any_child_is_published() {
    let mut model = Supervisor::new([1, 2], 0, 1, 2).unwrap();
    assert_eq!(model.admission_ready(), Err(abi::BUSY));
    model.request(0, model.launch(0).unwrap(), 99).unwrap();
    assert_eq!(model.admission_ready(), Ok(()));
    model.close(1); // An unrelated boot step ran first; peer exited meanwhile.
    assert_eq!(model.pending(), Some(99)); // Owner still needs its refusal result.
    assert_eq!(model.admission_ready(), Err(abi::PEER_CLOSED));
    model.failed();
    assert!(model.idle());
    assert!(!model.cancel_child());
    assert_eq!(
        model.request(0, model.launch(0).unwrap(), 0),
        Err(abi::PEER_CLOSED)
    );

    let mut model = Supervisor::new([3, 4], 0, 1, 2).unwrap();
    model.request(0, model.launch(0).unwrap(), 100).unwrap();
    model.close(0);
    assert_eq!(model.pending(), None);
    assert_eq!(model.admission_ready(), Err(abi::DENIED));
}

#[test]
fn loan_moves_launch_and_peer_and_returns_fresh_authority_after_reaping() {
    let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
    let original = model.launch(0).unwrap();
    let old_port = model.port(0).unwrap();
    model.can_lend(0, original, 1).unwrap();
    model.lend(1, [20, 21], [30, 31]);
    assert_eq!(model.launch(0), Err(abi::DENIED));
    assert_eq!(model.port(0), Err(abi::DENIED));
    assert_eq!(model.request(1, original, 0), Err(abi::BAD_HANDLE));
    model.request(1, model.launch(1).unwrap(), 0).unwrap();
    let old_child = model.started(4);
    model.close(1);
    model.reclaim();
    assert!(model.cancel_child());
    assert_eq!(model.launch(0), Err(abi::DENIED)); // Physical reap comes first.
    model.reaped([2; abi::EXIT_BYTES]);
    model.reclaim();
    assert_ne!(model.launch(0).unwrap(), original);
    assert_eq!(model.request(0, original, 0), Err(abi::BAD_HANDLE));
    assert_eq!(model.connect(0, old_port), Err(abi::BAD_HANDLE));
    assert_eq!(model.wait(0, old_child), Err(abi::BAD_HANDLE));
    model.request(0, model.launch(0).unwrap(), 0).unwrap();
    model.started(5);
}

#[test]
fn recursive_loan_skips_dead_lenders_and_clears_queued_admission() {
    let mut model = Supervisor::new([1, 2], 0, 0, 4).unwrap();
    model.lend(1, [20, 21], [30, 31]);
    model.lend(2, [40, 41], [50, 51]);
    model.request(2, model.launch(2).unwrap(), 99).unwrap();
    model.close(1);
    model.close(2);
    assert!(model.pending().is_none());
    model.reclaim();
    assert_eq!(model.owner, 0);
    assert_eq!(model.launch(1), Err(abi::DENIED));
    model.can_lend(0, model.launch(0).unwrap(), 1).unwrap();
    model.lend(1, [60, 61], [70, 71]);
    model.lend(2, [80, 81], [90, 91]);
    model.close(2);
    model.reclaim();
    assert_eq!(model.owner, 1); // A live immediate lender retains its budget.
    model.close(1);
    model.reclaim();
    assert_eq!(model.owner, 0);
}

#[test]
fn loans_reject_cycles_separate_peer_consent_and_busy_slots_without_mutation() {
    let split = Supervisor::new([1, 2], 0, 1, 3).unwrap();
    assert_eq!(
        split.can_lend(0, split.launch(0).unwrap(), 2),
        Err(abi::DENIED)
    );
    let mut model = Supervisor::new([1, 2], 0, 0, 3).unwrap();
    let grant = model.launch(0).unwrap();
    for target in [0, 3, 4, usize::MAX] {
        assert_eq!(model.can_lend(0, grant, target), Err(abi::DENIED));
    }
    model.request(0, grant, 0).unwrap();
    assert_eq!(model.can_lend(0, grant, 1), Err(abi::BUSY));
    model.started(100);
    assert_eq!(model.can_lend(0, grant, 1), Err(abi::BUSY));
    model.reaped([0; abi::EXIT_BYTES]);
    assert_eq!(model.can_lend(0, grant, 1), Err(abi::BUSY));
    model.consume();
    model.can_lend(0, grant, 1).unwrap();
}
#[test]
fn cancellation_is_owner_bound_preserves_outcome_and_cannot_hit_replacement() {
    let mut model = Supervisor::new([1, 2], 0, 1, 2).unwrap();
    let launch = model.launch(0).unwrap();
    model.request(0, launch, 0).unwrap();
    let first = model.started(100);
    assert_eq!(model.request_cancel(1, first), Err(abi::DENIED));
    assert_eq!(model.request_cancel(0, launch), Err(abi::BAD_HANDLE));
    assert!(!model.cancel_child());
    assert_eq!(model.request_cancel(0, first), Ok(true));
    assert!(model.cancel_child());
    model.reaped([2; abi::EXIT_BYTES]);
    assert!(!model.cancel_child());
    assert_eq!(model.request_cancel(0, first), Ok(false));
    assert_eq!(model.wait(0, first), Ok(Some([2; abi::EXIT_BYTES])));
    model.consume();
    model.request(0, launch, 0).unwrap();
    model.started(101);
    assert_eq!(model.request_cancel(0, first), Err(abi::BAD_HANDLE));
    assert!(!model.cancel_child());
}
#[test]
fn grants_are_bound_and_child_outcomes_must_be_consumed_before_reuse() {
    let mut model = Supervisor::new([1, 2], 0, 1, 2).unwrap();
    let launch = model.launch(0).unwrap();
    let port = model.port(1).unwrap();
    assert_eq!(model.request(1, launch, 0), Err(abi::DENIED));
    assert_eq!(model.request(0, 0, 0), Err(abi::BAD_HANDLE));
    model.request(0, launch, 7).unwrap();
    assert_eq!(model.pending(), Some(7));
    assert_eq!(model.request(0, launch, 0), Err(abi::BUSY));
    let first = model.started(100);
    assert_eq!(model.wait(0, first), Ok(None));
    assert_eq!(model.wait(1, first), Err(abi::DENIED));
    assert_eq!(model.connect(0, port), Err(abi::DENIED));
    model.connect(1, port).unwrap();
    assert_eq!(model.connect(1, port), Err(abi::BUSY));
    model.reaped([42; abi::EXIT_BYTES]);
    assert_eq!(model.wait(0, first), Ok(Some([42; abi::EXIT_BYTES])));
    assert_eq!(model.request(0, launch, 0), Err(abi::BUSY));
    model.consume();
    model.request(0, launch, 8).unwrap();
    let second = model.started(101);
    assert_ne!(first, second);
    assert_eq!(model.wait(0, first), Err(abi::BAD_HANDLE));
    model.connect(1, port).unwrap();
}
#[test]
fn failed_admission_can_retry_and_parent_death_cancels_live_child() {
    let mut model = Supervisor::new([1, 2], 0, 1, 2).unwrap();
    let grant = model.launch(0).unwrap();
    model.request(0, grant, 0).unwrap();
    model.failed();
    model.request(0, grant, 1).unwrap();
    model.started(100);
    model.close(0);
    assert!(model.cancel_child());
    assert_eq!(model.launch(0), Err(abi::DENIED));
    model.reaped([0; abi::EXIT_BYTES]);
    assert!(!model.cancel_child());
}
#[test]
fn dead_peer_and_invalid_boot_grants_fail_closed() {
    assert!(Supervisor::new([1, 2], 0, 2, 2).is_err());
    assert!(Supervisor::new([1, 2], 2, 1, 2).is_err());
    assert!(Supervisor::new([0, 0], 0, 1, 2).is_err());
    let mut model = Supervisor::new([1, 2], 0, 1, 2).unwrap();
    let grant = model.launch(0).unwrap();
    model.close(1);
    assert_eq!(model.request(0, grant, 0), Err(abi::PEER_CLOSED));
}

#[test]
fn initial_task_can_hold_distinct_launch_and_connection_grants() {
    let mut model = Supervisor::new([1, 2], 0, 0, 1).unwrap();
    let launch = model.launch(0).unwrap();
    let port = model.port(0).unwrap();
    assert_ne!(launch, port);
    assert_eq!(model.request(0, port, 0), Err(abi::BAD_HANDLE));
    model.request(0, launch, 0).unwrap();
    let child = model.started(100);
    assert_eq!(model.connect(0, launch), Err(abi::BAD_HANDLE));
    model.connect(0, port).unwrap();
    assert_eq!(model.launch(1), Err(abi::DENIED));
    assert_eq!(model.port(1), Err(abi::DENIED));
    model.request_cancel(0, child).unwrap();
    model.reaped([0; abi::EXIT_BYTES]);
    model.consume();
    model.request(0, launch, 1).unwrap();
    model.started(101);
    model.connect(0, port).unwrap();
    model.close(0);
    assert!(model.cancel_child());
    assert_eq!(model.port(0), Err(abi::DENIED));
}
