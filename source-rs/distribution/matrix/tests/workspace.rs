//! Execute the compiled Omega switch workflow, including late commit and failed restoration.
use crate::policy::{
    operation::{Ticket, Tickets},
    workspace::{Command as C, State},
};
fn start() -> (State, Ticket, Ticket) {
    let mut tickets = Tickets::default();
    let t = tickets.issue().unwrap();
    (State::new(t), t, tickets.issue().unwrap())
}
fn issue(s: &mut State, c: C) {
    assert_eq!(s.command(), Some(c));
    assert_eq!(s.command(), None);
}
fn complete(s: &mut State, t: Ticket, c: C, status: u64, committed: bool) {
    assert!(s.complete(t, c, status, committed));
    assert!(!s.complete(t, c, status, committed));
}
fn step(s: &mut State, t: Ticket, c: C, status: u64, committed: bool) {
    issue(s, c);
    complete(s, t, c, status, committed);
}
#[test]
fn only_the_current_ticket_and_command_can_publish_activation() {
    let (mut s, t, wrong) = start();
    assert!(!s.complete(t, C::Begin, 0, false));
    step(&mut s, t, C::Begin, 0, false);
    issue(&mut s, C::Wait);
    assert!(!s.complete(wrong, C::Wait, 0, true));
    assert!(!s.complete(t, C::WaitRestore, 0, true));
    assert_eq!(s.command(), None);
    complete(&mut s, t, C::Wait, 0, true);
    step(&mut s, t, C::Activate, 0, false);
    step(&mut s, t, C::Finish, 0, false);
    assert_eq!(s.status(), 0);
    assert_eq!(s.command(), None);
    assert!(!s.cancel(13));
}
#[test]
fn refusal_before_scene_and_failed_begin_finish_without_activation() {
    for cancel in [false, true] {
        let (mut s, t, _) = start();
        if cancel {
            assert!(s.cancel(13));
            assert!(!s.cancel(14));
        }
        step(&mut s, t, C::Begin, 5, false);
        step(&mut s, t, C::Finish, 0, false);
        assert_eq!(s.status(), if cancel { 13 } else { 5 });
    }
}
#[test]
fn cancellation_drains_scene_and_restores_only_if_it_committed() {
    for committed in [false, true] {
        for refused in [false, true] {
            let (mut s, t, _) = start();
            step(&mut s, t, C::Begin, 0, false);
            issue(&mut s, C::Wait);
            if refused {
                assert!(!s.cancel(0));
                assert!(s.cancel(13));
                assert!(!s.cancel(14));
                assert_eq!(
                    s.command(),
                    None,
                    "cancellation must drain the outstanding scene"
                );
            }
            complete(&mut s, t, C::Wait, 5, committed);
            if committed {
                assert!(!s.cancel(14), "restoration cannot be cancelled");
                step(&mut s, t, C::Restore, 0, false);
                issue(&mut s, C::WaitRestore);
                assert!(
                    !s.complete(t, C::Wait, 0, true),
                    "old commit is not restoration"
                );
                assert!(!s.cancel(14));
                complete(&mut s, t, C::WaitRestore, 0, true);
            }
            step(&mut s, t, C::Finish, 0, false);
            assert_eq!(s.status(), if refused { 13 } else { 5 });
        }
    }
}
#[test]
fn successful_commit_after_refusal_restores_instead_of_activating() {
    let (mut s, t, _) = start();
    step(&mut s, t, C::Begin, 0, false);
    issue(&mut s, C::Wait);
    assert!(s.cancel(13));
    complete(&mut s, t, C::Wait, 0, true);
    step(&mut s, t, C::Restore, 0, false);
    step(&mut s, t, C::WaitRestore, 0, true);
    step(&mut s, t, C::Finish, 0, false);
    assert_eq!(s.status(), 13);
}
#[test]
fn failed_restore_cannot_report_finished_or_release_custody() {
    for begin_failure in [false, true] {
        for committed in [false, true] {
            for status in [0, 5] {
                if !begin_failure && committed && status == 0 {
                    continue;
                }
                let (mut s, t, _) = start();
                step(&mut s, t, C::Begin, 0, false);
                step(&mut s, t, C::Wait, 13, true);
                step(
                    &mut s,
                    t,
                    C::Restore,
                    if begin_failure { 5 } else { 0 },
                    false,
                );
                if !begin_failure {
                    step(&mut s, t, C::WaitRestore, status, committed);
                }
                issue(&mut s, C::Failed);
                assert!(!s.complete(t, C::Finish, 0, false));
                assert!(!s.cancel(14));
                assert_eq!(s.status(), 13);
            }
        }
    }
}
#[test]
fn failed_local_activation_does_not_report_success() {
    let (mut s, t, _) = start();
    step(&mut s, t, C::Begin, 0, false);
    step(&mut s, t, C::Wait, 0, true);
    step(&mut s, t, C::Activate, 5, false);
    issue(&mut s, C::Failed);
    assert_eq!(s.status(), 5);
}
