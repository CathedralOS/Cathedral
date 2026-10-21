use super::*;
#[test]
fn impossible_scheduler_capacity_is_a_recoverable_error() {
    assert!(Scheduler::try_new(usize::MAX).is_err());
    assert!(Scheduler::try_new(0).unwrap().finished());
}
#[test]
fn nonblocking_operations_keep_the_quantum_but_timer_and_yield_rotate() {
    let mut scheduler = populated(2);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(0));
    for _ in 0..100 {
        assert_eq!(scheduler.advance(Event::Continue, 0, true), Some(0));
    }
    assert_eq!(scheduler.advance(Event::Timer, 1, true), Some(1));
    assert_eq!(scheduler.advance(Event::Continue, 1, true), Some(1));
    assert_eq!(scheduler.advance(Event::Yield, 1, true), Some(0));
    assert_eq!(scheduler.advance(Event::Block, 1, true), Some(1));
    scheduler.unblock(0);
    assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
}
#[test]
fn cancelling_a_waiter_requires_reaping_before_reuse() {
    let mut scheduler = populated(2);
    let old = scheduler.id(0);
    scheduler.advance(Event::Yield, 0, true);
    scheduler.advance(Event::Block, 1, true);
    scheduler.park(Event::Yield, 2);
    scheduler.cancel(0);
    scheduler.unblock(0);
    assert!(!scheduler.is_alive(old));
    assert_eq!(scheduler.states()[0], TaskState::Exited);
    scheduler.reap(0);
    let new = scheduler.admit(0);
    assert_ne!(old, new);
    assert!(!scheduler.is_alive(old));
    assert!(scheduler.is_alive(new));
}
#[test]
fn blocked_tasks_need_explicit_wakeup_and_keep_their_identity() {
    let mut scheduler = populated(2);
    let waiter = scheduler.id(0);
    scheduler.advance(Event::Yield, 0, true);
    assert_eq!(scheduler.advance(Event::Block, 1, true), Some(1));
    assert!(scheduler.is_alive(waiter));
    assert_eq!(scheduler.advance(Event::Timer, 100, true), Some(1));
    assert_eq!(scheduler.states()[0], TaskState::Blocked);
    scheduler.unblock(0);
    scheduler.unblock(0);
    assert_eq!(scheduler.advance(Event::Yield, 101, true), Some(0));
    scheduler.advance(Event::Exit, 102, true);
    scheduler.unblock(0);
    assert_eq!(scheduler.states()[0], TaskState::Exited);
}
fn populated(limit: usize) -> Scheduler {
    let mut scheduler = Scheduler::new(limit);
    for slot in 0..limit {
        scheduler.admit(slot);
    }
    scheduler
}
#[test]
fn round_robin_and_cooperative_timer_have_distinct_behavior() {
    let mut scheduler = populated(2);
    assert_eq!(scheduler.advance(Event::Yield, 0, false), Some(0));
    assert_eq!(scheduler.advance(Event::Timer, 1, false), Some(0));
    assert_eq!(scheduler.advance(Event::Yield, 1, false), Some(1));
    assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
    assert_eq!(scheduler.states(), &[TaskState::Running, TaskState::Ready]);
}
#[test]
fn sleeping_tasks_leave_cpu_idle_until_deadline_and_exits_are_terminal() {
    let mut scheduler = populated(2);
    scheduler.advance(Event::Yield, 0, false);
    assert_eq!(scheduler.advance(Event::Sleep(5), 0, false), Some(1));
    assert_eq!(scheduler.advance(Event::Sleep(10), 0, false), None);
    assert_eq!(scheduler.advance(Event::Timer, 4, true), None);
    assert_eq!(scheduler.advance(Event::Timer, 5, true), Some(0));
    assert_eq!(scheduler.advance(Event::Exit, 5, true), None);
    assert_eq!(scheduler.advance(Event::Timer, 10, true), Some(1));
    assert_eq!(scheduler.advance(Event::Exit, 10, true), None);
    assert!(scheduler.finished());
    assert_eq!(scheduler.advance(Event::Timer, 11, true), None);
}
#[test]
fn zero_sleep_yields_and_deadlines_survive_clock_wrap() {
    let mut scheduler = populated(1);
    scheduler.advance(Event::Yield, u64::MAX - 2, true);
    assert_eq!(
        scheduler.advance(Event::Sleep(0), u64::MAX - 2, true),
        Some(0)
    );
    assert_eq!(scheduler.advance(Event::Sleep(5), u64::MAX - 2, true), None);
    assert_eq!(scheduler.advance(Event::Timer, 1, true), None);
    assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
    assert!(Scheduler::new(0).finished());
}

#[test]
fn admission_requires_reaping_and_reused_slots_reject_stale_ids() {
    let mut scheduler = Scheduler::new(2);
    let first = scheduler.admit(scheduler.vacancy().unwrap());
    let survivor = scheduler.admit(scheduler.vacancy().unwrap());
    assert!(scheduler.vacancy().is_none());
    scheduler.advance(Event::Yield, 0, true);
    scheduler.park(Event::Exit, 1);
    assert!(!scheduler.is_alive(first));
    assert!(scheduler.vacancy().is_none());
    scheduler.reap(first.slot);
    let replacement = scheduler.admit(scheduler.vacancy().unwrap());
    assert_eq!(first.slot, replacement.slot);
    assert_ne!(first, replacement);
    assert!(!scheduler.is_alive(first));
    assert!(scheduler.is_alive(replacement));
    assert!(scheduler.is_alive(survivor));
    assert_eq!(
        scheduler.advance(Event::Yield, 1, true),
        Some(survivor.slot)
    );
}

#[test]
fn cancelled_sleep_cannot_wake_a_replacement_and_equal_deadlines_all_wake() {
    let mut scheduler = populated(3);
    scheduler.advance(Event::Yield, 0, true);
    scheduler.advance(Event::Sleep(5), 0, true);
    scheduler.advance(Event::Sleep(5), 0, true);
    scheduler.advance(Event::Sleep(10), 0, true);
    scheduler.cancel(0);
    scheduler.reap(0);
    scheduler.admit(0);
    scheduler.advance(Event::Yield, 1, true);
    scheduler.advance(Event::Sleep(20), 1, true);
    assert_eq!(scheduler.advance(Event::Timer, 5, true), Some(1));
    assert_eq!(scheduler.states()[0], TaskState::Sleeping { deadline: 21 });
    assert_eq!(scheduler.advance(Event::Exit, 10, true), Some(2));
    assert_eq!(scheduler.advance(Event::Exit, 20, true), None);
    assert_eq!(scheduler.advance(Event::Timer, 21, true), Some(0));

    let mut scheduler = populated(3);
    scheduler.advance(Event::Yield, 0, true);
    for _ in 0..3 {
        scheduler.advance(Event::Sleep(10), 0, true);
    }
    scheduler.advance(Event::Timer, 10, true);
    assert_eq!(
        scheduler.states(),
        &[TaskState::Running, TaskState::Ready, TaskState::Ready]
    );
}

#[test]
fn dormant_capacity_does_not_scale_dispatch_accounting_or_timer_inspections() {
    fn workload(capacity: usize) -> (usize, usize, usize) {
        let mut scheduler = Scheduler::new(capacity);
        scheduler.set_parent(2, 0);
        for slot in 0..3 {
            scheduler.admit(slot);
        }
        scheduler.account_cpu(0);
        for now in 0..100 {
            scheduler.advance(Event::Timer, now, true);
            scheduler.account_cpu((now + 1) * 100);
            assert!(!scheduler.finished());
            assert!(scheduler.has_eligible_ready());
        }
        (
            scheduler.shares.inspections,
            scheduler.budgets.visits,
            scheduler.sleepers.inspections,
        )
    }
    assert_eq!(workload(3), workload(50_000));
}

#[test]
fn future_sleepers_require_only_one_deadline_inspection_per_dispatch() {
    let mut scheduler = populated(10_000);
    scheduler.advance(Event::Yield, 0, true);
    for _ in 0..10_000 {
        scheduler.advance(Event::Sleep(1_000), 0, true);
    }
    let before = scheduler.sleepers.inspections;
    for tick in 1..100 {
        assert_eq!(scheduler.advance(Event::Timer, tick, true), None);
    }
    assert_eq!(scheduler.sleepers.inspections - before, 99);
}
