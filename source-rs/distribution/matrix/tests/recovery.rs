//! Compiled workflow tests: command issuance, completion custody and bounded retries.
use crate::policy::recovery::{Action, State, Ticket, Tickets};
fn begin(s: &mut State, tickets: &mut Tickets) -> Ticket {
    assert_eq!(s.action(true, true), Action::Read);
    assert_eq!(s.action(true, true), Action::Wait);
    let ticket = tickets.issue().unwrap();
    s.read_started(ticket);
    ticket
}
fn reading(tickets: &mut Tickets) -> (State, Ticket) {
    let mut s = State::configured();
    assert!(s.retired(true));
    let ticket = begin(&mut s, tickets);
    (s, ticket)
}
fn admitted(s: &mut State, ticket: Ticket) {
    assert!(s.read_completed(ticket, true));
    assert_eq!(s.action(true, true), Action::Admit(ticket));
    assert_eq!(s.action(true, true), Action::Wait);
    assert!(s.admission_finished(ticket, true));
}
fn fallback(tickets: &mut Tickets) -> State {
    let (mut s, t) = reading(tickets);
    assert!(s.read_completed(t, false));
    assert_eq!(s.action(true, true), Action::Fallback);
    assert!(s.fallback_finished(true));
    s
}
#[test]
fn automatic_retry_is_consumed_and_manual_attempts_never_rearm_it() {
    let mut tickets = Tickets::default();
    let (mut s, t) = reading(&mut tickets);
    admitted(&mut s, t);
    assert!(!s.ready(false));
    for _ in 0..16 {
        assert!(s.retired(true));
        assert_eq!(s.action(true, true), Action::Fallback);
        assert!(s.fallback_finished(true));
        assert!(!s.request(false));
        assert!(s.request(true));
        assert!(!s.request(true));
        assert!(!s.ready(true));
        assert_eq!(s.action(true, false), Action::RetireFallback);
        s.explicit_retired();
        let t = begin(&mut s, &mut tickets);
        admitted(&mut s, t);
    }
}
#[test]
fn commands_wait_for_matching_completions_and_reject_duplicates() {
    let mut tickets = Tickets::default();
    let (mut s, t) = reading(&mut tickets);
    let wrong = tickets.issue().unwrap();
    for _ in 0..16 {
        assert_eq!(s.action(true, true), Action::Wait);
        assert!(!s.read_completed(wrong, true));
        assert!(!s.admission_finished(t, true));
    }
    assert!(s.read_completed(t, true));
    assert!(!s.read_completed(t, false)); // A duplicate error cannot erase the image.
    assert_eq!(s.action(true, true), Action::Admit(t));
    assert!(!s.admission_finished(wrong, true));
    assert_eq!(s.action(true, true), Action::Wait);
    assert!(s.admission_finished(t, true));
    assert!(!s.admission_finished(t, false));
    assert!(!s.read_completed(t, true));
}
#[test]
fn cancelled_and_replacement_work_cannot_consume_late_images() {
    let mut tickets = Tickets::default();
    let (mut old, t1) = reading(&mut tickets);
    assert_eq!(old.action(false, true), Action::Cancel);
    assert!(!old.read_completed(t1, true));
    let (mut new, t2) = reading(&mut tickets);
    assert_ne!(t1, t2);
    assert!(!new.read_completed(t1, true));
    assert!(!new.read_completed(t1, false));
    assert_eq!(new.action(true, true), Action::Wait);
    assert!(new.read_completed(t2, true));
    // A replacement or revocation after completion discards admission too.
    assert_eq!(new.action(true, false), Action::Cancel);
    assert!(!new.admission_finished(t2, true));
    assert_eq!(new.action(true, true), Action::Wait);
}
#[test]
fn clean_exit_stop_and_fallback_failure_do_not_restart() {
    let mut tickets = Tickets::default();
    let mut states = [
        State::configured(),
        reading(&mut tickets).0,
        fallback(&mut tickets),
    ];
    for s in &mut states {
        assert!(!s.retired(false));
        assert!(!s.ready(false));
        assert!(!s.request(true));
        assert_eq!(s.action(true, true), Action::Wait);
    }
    let mut s = fallback(&mut tickets);
    assert!(!s.retired(true));
    assert!(!s.ready(false));
    let (mut s, t) = reading(&mut tickets);
    assert!(s.read_completed(t, false));
    assert!(!s.fallback_finished(false));
    assert!(!s.ready(false));
}
#[test]
fn pending_work_waits_and_authority_changes_cancel_every_stage() {
    for stage in 0..6 {
        for (authority, vacant) in [(false, true), (true, false)] {
            let mut tickets = Tickets::default();
            let mut s = State::configured();
            assert!(s.retired(true));
            let mut ticket = None;
            if stage >= 1 {
                ticket = Some(begin(&mut s, &mut tickets));
            }
            if stage == 2 {
                assert!(s.read_completed(ticket.unwrap(), true));
            }
            if stage >= 3 {
                assert!(s.read_completed(ticket.unwrap(), false));
            }
            if stage >= 4 {
                assert!(s.fallback_finished(true));
                assert!(s.request(true));
            }
            if stage == 5 {
                s.explicit_retired();
            }
            assert!(s.ready(false));
            assert!(!s.ready(true));
            assert!(s.ready(false));
            if stage == 4 && authority {
                assert_eq!(s.action(authority, vacant), Action::RetireFallback);
            } else {
                assert_eq!(s.action(authority, vacant), Action::Cancel);
                if let Some(t) = ticket {
                    assert!(!s.read_completed(t, true));
                }
            }
        }
    }
}
#[test]
fn initial_fallback_supply_preserves_only_a_running_retry_budget() {
    let mut tickets = Tickets::default();
    let mut initial = State::configured();
    assert!(initial.fallback_changed(true));
    assert!(initial.retired(true));
    let (mut spent, t) = reading(&mut tickets);
    admitted(&mut spent, t);
    assert!(spent.fallback_changed(true));
    assert!(spent.retired(true));
    assert_eq!(spent.action(true, true), Action::Fallback);
    for mut pending in [
        initial,
        reading(&mut tickets).0,
        spent,
        fallback(&mut tickets),
    ] {
        assert!(!pending.fallback_changed(true));
        assert!(!pending.ready(false));
    }
    let (mut s, t) = reading(&mut tickets);
    assert!(!s.fallback_changed(false));
    assert!(!s.read_completed(t, true));
}
#[test]
fn unavailable_read_or_failed_admission_goes_to_fallback_once() {
    let mut s = State::configured();
    assert!(s.retired(true));
    assert_eq!(s.action(true, true), Action::Read);
    s.read_unavailable();
    assert_eq!(s.action(true, true), Action::Fallback);
    let mut tickets = Tickets::default();
    let (mut s, t) = reading(&mut tickets);
    assert!(s.read_completed(t, true));
    assert_eq!(s.action(true, true), Action::Admit(t));
    assert!(s.admission_finished(t, false));
    assert_eq!(s.action(true, true), Action::Fallback);
    assert!(s.fallback_finished(true));
    assert!(!s.request(false));
    assert!(s.request(true));
    assert!(!s.request(true));
    assert!(!s.fallback_changed(false));
    assert_eq!(s.action(true, true), Action::Wait);
}
