use crate::scheduler::{Event, Scheduler, TaskState};

fn tree(parents: &[Option<usize>]) -> Scheduler {
    let mut scheduler = Scheduler::new(parents.len());
    for (child, &parent) in parents.iter().enumerate() {
        if let Some(parent) = parent {
            scheduler.set_parent(child, parent);
        }
        scheduler.admit(child);
    }
    scheduler
}
#[test]
fn descendants_subdivide_turns_without_diluting_the_outside_sibling() {
    // Two root accounts; 0 contains 2 and 3, and 2 contains 4 and 5.
    let mut scheduler = tree(&[None, None, Some(0), Some(0), Some(2), Some(2)]);
    let mut turns = [0; 6];
    for now in 0..360 {
        turns[scheduler.advance(Event::Timer, now, true).unwrap()] += 1;
    }
    assert_eq!(turns, [60, 180, 20, 60, 20, 20]);
}
#[test]
fn blocking_lends_idle_capacity_without_banking_wakeup_credit() {
    let mut scheduler = tree(&[None, None, Some(0), Some(0)]);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(0));
    assert_eq!(scheduler.advance(Event::Block, 0, true), Some(1));
    assert_eq!(scheduler.advance(Event::Block, 0, true), Some(2));
    assert_eq!(scheduler.advance(Event::Block, 0, true), Some(3));
    for tick in 1..100 {
        assert_eq!(scheduler.advance(Event::Timer, tick, true), Some(3));
    }
    scheduler.unblock(1);
    assert_eq!(scheduler.advance(Event::Timer, 100, true), Some(1));
    assert_eq!(scheduler.advance(Event::Timer, 101, true), Some(3));
    assert_eq!(scheduler.advance(Event::Block, 101, true), Some(1));
    assert_eq!(scheduler.advance(Event::Block, 101, true), None);
}
#[test]
fn yield_and_syscall_activity_do_not_create_extra_outer_turns() {
    let mut scheduler = tree(&[None, None, Some(0)]);
    for round in 0..100 {
        let selected = scheduler.advance(Event::Yield, round, true).unwrap();
        assert_eq!(selected == 1, round % 2 == 1);
        for _ in 0..8 {
            assert_eq!(
                scheduler.advance(Event::Continue, round, true),
                Some(selected)
            );
        }
    }
}
#[test]
fn restart_keeps_the_outer_cursor_and_loan_return_restores_the_owner() {
    let mut scheduler = tree(&[None, None, Some(0)]);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(0));
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(1));
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(2));
    scheduler.park(Event::Exit, 0);
    let old = scheduler.id(2);
    scheduler.reap(2);
    scheduler.admit(2);
    assert!(!scheduler.is_alive(old));
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(1));
    scheduler.park(Event::Yield, 0);
    scheduler.cancel(2);
    scheduler.reap(2);
    scheduler.set_parent(2, 1); // An idle loan changes which envelope pays.
    scheduler.admit(2);
    let mut turns = [0; 3];
    for tick in 0..120 {
        turns[scheduler.advance(Event::Timer, tick, true).unwrap()] += 1;
    }
    assert_eq!(turns, [60, 30, 30]);
    scheduler.park(Event::Yield, 120);
    scheduler.cancel(2);
    scheduler.reap(2);
    scheduler.set_parent(2, 0);
    scheduler.admit(2);
    turns.fill(0);
    for tick in 120..240 {
        turns[scheduler.advance(Event::Timer, tick, true).unwrap()] += 1;
    }
    assert_eq!(turns, [30, 60, 30]);
}
#[test]
fn dormant_and_failed_admission_do_not_contend_for_turns() {
    let mut scheduler = Scheduler::new(4);
    scheduler.set_parent(2, 0);
    scheduler.set_parent(3, 2);
    scheduler.admit(0);
    scheduler.admit(1);
    for tick in 0..40 {
        assert_eq!(
            scheduler.advance(Event::Timer, tick, true),
            Some(tick as usize % 2)
        );
    }
    assert_eq!(&scheduler.states()[2..], &[TaskState::Vacant; 2]);
}
