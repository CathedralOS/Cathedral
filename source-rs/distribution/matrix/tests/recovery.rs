//! Run the compiled Omega retry policy; no Rust mirror of its phase encoding.
use crate::policy::recovery::{Action, State};
fn reading() -> State {
    let mut s = State::configured();
    assert!(s.retired(true));
    assert_eq!(s.action(true, true, false), Action::Read);
    s.read_started();
    s
}
fn fallback() -> State {
    let mut s = reading();
    s.read_finished(false);
    assert_eq!(s.action(true, true, true), Action::Fallback);
    assert!(s.fallback_finished(true));
    s
}
#[test]
fn automatic_retry_is_consumed_and_manual_attempts_never_rearm_it() {
    let mut s = reading();
    assert_eq!(s.action(true, true, false), Action::Wait);
    assert_eq!(s.action(true, true, true), Action::Admit);
    s.read_finished(true);
    assert!(!s.ready(false));
    for _ in 0..16 {
        assert!(s.retired(true));
        assert_eq!(s.action(true, true, false), Action::Fallback);
        assert!(s.fallback_finished(true));
        assert!(!s.request(false));
        assert!(!s.ready(false));
        assert!(s.request(true));
        assert!(!s.request(true));
        assert!(!s.ready(true));
        assert_eq!(s.action(true, false, false), Action::RetireFallback);
        s.explicit_retired();
        assert_eq!(s.action(true, true, false), Action::Read);
        s.read_started();
        s.read_finished(true);
    }
}
#[test]
fn clean_exit_stop_and_fallback_failure_do_not_restart() {
    let mut states = [State::configured(), reading(), fallback()];
    for s in &mut states {
        assert!(!s.retired(false));
        assert!(!s.ready(false));
        assert!(!s.request(true));
        assert_eq!(s.action(true, true, true), Action::Wait);
    }
    let mut s = fallback();
    assert!(!s.retired(true));
    assert!(!s.ready(false));
    let mut s = reading();
    s.read_finished(false);
    assert!(!s.fallback_finished(false));
    assert!(!s.ready(false));
    assert!(!s.request(true));
}
#[test]
fn pending_work_waits_for_custody_and_cannot_use_changed_authority_or_slot() {
    for stage in 0..4 {
        let mut s = State::configured();
        assert!(s.retired(true));
        if stage >= 1 {
            s.read_started();
        }
        if stage >= 2 {
            s.read_finished(false);
        }
        if stage == 3 {
            assert!(s.fallback_finished(true));
            assert!(s.request(true));
        }
        assert!(s.ready(false));
        assert!(!s.ready(true));
        for ready in [false, true] {
            for vacant in [false, true] {
                assert_eq!(s.action(false, vacant, ready), Action::Cancel);
            }
        }
        if stage != 3 {
            assert_eq!(s.action(true, false, true), Action::Cancel);
        }
        // Replacement/revocation cancels even an already acknowledged manual request.
        assert!(!s.fallback_changed(false));
        assert!(!s.ready(false));
        assert!(!s.request(true));
    }
}
#[test]
fn initial_fallback_supply_preserves_only_a_running_retry_budget() {
    let mut initial = State::configured();
    assert!(initial.fallback_changed(true));
    assert!(initial.retired(true));
    assert_eq!(initial.action(true, true, false), Action::Read);
    let mut spent = reading();
    spent.read_finished(true);
    assert!(spent.fallback_changed(true));
    assert!(spent.retired(true));
    assert_eq!(spent.action(true, true, false), Action::Fallback);
    for mut pending in [initial, reading(), spent, fallback()] {
        assert!(!pending.fallback_changed(true));
        assert!(!pending.ready(false));
    }
}
#[test]
fn retry_request_requires_current_authenticated_fallback() {
    let mut configured = State::configured();
    assert!(!configured.request(true));
    let mut reading = reading();
    assert!(!reading.request(true));
    let mut s = fallback();
    assert!(!s.request(false));
    assert!(s.request(true));
    assert!(!s.request(true));
    assert_eq!(s.action(false, false, false), Action::Cancel);
}
