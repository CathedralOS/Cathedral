//! Execute the compiled Omega launch workflow, including each partial failure.
use crate::policy::{
    launch::{Command as C, State},
    operation::{Ticket, Tickets},
};
fn start(selection: bool, child: bool, desktop: bool) -> (State, Ticket, Ticket) {
    let mut tickets = Tickets::default();
    let t = tickets.issue().unwrap();
    (
        State::new(t, selection, child, desktop),
        t,
        tickets.issue().unwrap(),
    )
}
fn step(s: &mut State, t: Ticket, c: C, storage: bool) {
    assert_eq!(s.command(), Some(c));
    assert_eq!(s.command(), None);
    assert!(s.complete(t, c, 0, storage));
    assert!(!s.complete(t, c, 0, storage));
}
#[test]
fn launch_shapes_skip_only_inapplicable_work_and_finish_once() {
    for selection in [false, true] {
        for child in [false, true] {
            for desktop in [false, true] {
                for storage in [false, true] {
                    let (mut s, t, wrong) = start(selection, child, desktop);
                    assert!(!s.complete(t, C::Read, 0, false));
                    assert_eq!(s.command(), Some(C::Read));
                    assert!(!s.complete(wrong, C::Read, 0, false));
                    assert!(!s.complete(t, C::Admit, 0, false));
                    assert_eq!(s.command(), None);
                    assert!(s.complete(t, C::Read, 0, false));
                    if selection {
                        step(&mut s, t, C::RetirePicker, false);
                    }
                    step(&mut s, t, C::Admit, storage);
                    if storage {
                        step(&mut s, t, C::Bind, false);
                    }
                    if child {
                        step(&mut s, t, C::Place, false);
                    }
                    if desktop {
                        step(&mut s, t, C::Activate, false);
                    }
                    step(&mut s, t, C::Publish, false);
                    step(&mut s, t, C::Finish, false);
                    assert_eq!(s.command(), None);
                    assert_eq!(s.status(), 0);
                }
            }
        }
    }
}
#[test]
fn each_partial_failure_reclaims_task_and_restores_retired_picker() {
    let commands = [
        C::Read,
        C::RetirePicker,
        C::Admit,
        C::Bind,
        C::Place,
        C::Activate,
        C::Publish,
    ];
    for (at, failed) in commands.iter().enumerate() {
        let (mut s, t, _) = start(true, true, true);
        for c in &commands[..at] {
            step(&mut s, t, *c, true);
        }
        assert_eq!(s.command(), Some(*failed));
        assert!(s.complete(t, *failed, 99, false));
        if at > 2 {
            step(&mut s, t, C::Rollback, false);
        }
        if at > 1 {
            step(&mut s, t, C::RestorePicker, false);
        }
        step(&mut s, t, C::Finish, false);
        assert_eq!(s.status(), 99);
        assert_eq!(s.command(), None);
    }
}
#[test]
fn cancelled_read_cannot_complete_or_be_reinterpreted_by_replacement() {
    let mut tickets = Tickets::default();
    let old = tickets.issue().unwrap();
    let new = tickets.issue().unwrap();
    let mut a = State::new(old, false, true, true);
    assert_eq!(a.command(), Some(C::Read));
    a.cancel(55);
    assert!(!a.complete(old, C::Read, 0, false));
    step(&mut a, old, C::Finish, false);
    let mut b = State::new(new, false, true, true);
    assert_eq!(b.command(), Some(C::Read));
    assert!(!b.complete(old, C::Read, 0, false));
    assert!(b.complete(new, C::Read, 0, false));
    step(&mut b, new, C::Admit, true);
    b.cancel(66);
    step(&mut b, new, C::Rollback, false);
    step(&mut b, new, C::Finish, false);
    assert_eq!(b.status(), 66);
}

#[test]
fn pending_binding_accepts_only_its_live_completion_and_cancel_requires_rollback() {
    let (mut s, t, wrong) = start(false, true, true);
    step(&mut s, t, C::Read, false);
    step(&mut s, t, C::Admit, true);
    assert!(!s.accepts(t, C::Bind));
    assert_eq!(s.command(), Some(C::Bind));
    for _ in 0..8 {
        assert!(s.accepts(t, C::Bind));
        assert!(!s.accepts(wrong, C::Bind));
        assert!(!s.accepts(t, C::Read));
        assert_eq!(s.command(), None);
    }
    s.cancel(44);
    assert!(!s.accepts(t, C::Bind));
    assert!(!s.complete(t, C::Bind, 0, false));
    step(&mut s, t, C::Rollback, false);
    step(&mut s, t, C::Finish, false);
}
