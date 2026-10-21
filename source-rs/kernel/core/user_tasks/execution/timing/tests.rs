use super::waits::Observations;
use crate::user_tasks::execution::waits::Kind;

#[test]
fn cpu_ceiling_can_delay_selection_after_a_wait_is_ready() {
    use crate::scheduler::{Event, Scheduler, TaskState};
    let mut scheduler = Scheduler::new(1);
    let mut trace = Observations::try_new(1).unwrap();
    assert!(scheduler.configure_cpu(&[20_000]));
    scheduler.account_cpu(0);
    scheduler.admit(0);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(0));
    scheduler.account_cpu(35_000);
    scheduler.park(Event::Block, 0);
    scheduler.unblock(0);
    trace.complete(0, Kind::Receive, None, 0, 35_000);
    assert_eq!(scheduler.states()[0], TaskState::Ready);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), None);
    assert!(trace.pending(0));
    scheduler.account_cpu(75_001);
    let selected = scheduler.advance(Event::Yield, 0, true).unwrap();
    trace.select(selected, 75_001);
    let sample = trace.sample(0, Kind::Receive);
    assert_eq!(
        (sample.selected, sample.total_ready_us, sample.due),
        (1, 40_001, 0)
    );
}

#[test]
fn first_selection_separates_ready_delay_from_deadline_lateness() {
    let mut trace = Observations::try_new(2).unwrap();
    trace.complete(0, Kind::Readiness, Some(10), 13, 100);
    assert!(trace.pending(0));
    trace.select(1, 200); // A peer's turn does not finish this observation.
    trace.select(0, 350);
    trace.select(0, 500); // Later turns cannot count the same wake twice.
    let sample = trace.sample(0, Kind::Readiness);
    assert_eq!(
        (sample.completed, sample.selected, sample.cancelled),
        (1, 1, 0)
    );
    assert_eq!((sample.total_ready_us, sample.max_ready_us), (250, 250));
    assert_eq!(
        (sample.due, sample.total_late_ticks, sample.max_late_ticks),
        (1, 3, 3)
    );
    assert!(!trace.pending(0));
}

#[test]
fn cancellation_and_slot_reuse_cannot_inherit_a_ready_timestamp() {
    let mut trace = Observations::try_new(1).unwrap();
    trace.complete(0, Kind::Receive, None, 0, 0); // Zero is a valid timestamp.
    trace.cancel(0);
    trace.cancel(0);
    trace.select(0, 900);
    trace.complete(0, Kind::Child, Some(20), 10, 1_000);
    trace.select(0, 1_100);
    let old = trace.sample(0, Kind::Receive);
    assert_eq!((old.completed, old.selected, old.cancelled), (1, 0, 1));
    let new = trace.sample(0, Kind::Child);
    assert_eq!((new.selected, new.total_ready_us, new.due), (1, 100, 0));
}

#[test]
fn deadline_ties_and_wrapping_ticks_keep_separate_units() {
    let mut trace = Observations::try_new(1).unwrap();
    for (deadline, now, start, selected) in [(u64::MAX, 2, 10, 20), (4, 4, 100, 100)] {
        trace.complete(0, Kind::Readiness, Some(deadline), now, start);
        trace.select(0, selected);
    }
    let sample = trace.sample(0, Kind::Readiness);
    assert_eq!(
        (sample.due, sample.total_late_ticks, sample.max_late_ticks),
        (2, 3, 3)
    );
    assert_eq!(
        (sample.selected, sample.total_ready_us, sample.max_ready_us),
        (2, 10, 10)
    );
}
