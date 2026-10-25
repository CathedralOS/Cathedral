//! Exercise the compiled startup policy, including every retirement fact combination.
use crate::policy::startup::{Context, Picker, Role, State};
fn context() -> Context {
    Context {
        login: true,
        entered: false,
        desktop: false,
        owner_recovers: false,
    }
}
#[test]
fn retirement_classifies_roles_and_leaves_unrelated_children_alone() {
    for bits in 0..32 {
        for role in [Role::Picker, Role::User, Role::Background, Role::Other] {
            let mut s = State::default();
            let failed = bits & 1 != 0;
            let login = bits & 2 != 0;
            let entered = bits & 4 != 0;
            let desktop = bits & 8 != 0;
            let owner_recovers = bits & 16 != 0;
            let local = failed && !(login && owner_recovers);
            let user_failed = local && login && entered && matches!(role, Role::User);
            let picker_failed = local && login && !entered && matches!(role, Role::Picker);
            let background_failed = local && desktop && matches!(role, Role::Background);
            assert_eq!(
                s.retired(
                    role,
                    failed,
                    Context {
                        login,
                        entered,
                        desktop,
                        owner_recovers
                    }
                ),
                user_failed
            );
            assert_eq!(
                s.take_picker(false),
                if user_failed || picker_failed {
                    Picker::Restart
                } else {
                    Picker::Wait
                }
            );
            assert_eq!(s.take_background(false), background_failed);
        }
    }
}
#[test]
fn picker_is_bounded_busy_preserves_pending_and_exhaustion_is_consumed() {
    let mut s = State::default();
    assert_eq!(s.take_picker(false), Picker::Wait);
    for expected in [Picker::Restart, Picker::Exhausted] {
        assert_eq!(s.picker_started(), 1);
        assert!(!s.retired(Role::Picker, true, context()));
        for _ in 0..4 {
            assert_eq!(s.take_picker(true), Picker::Wait);
        }
        assert_eq!(s.take_picker(false), expected);
        assert_eq!(s.take_picker(false), Picker::Wait);
    }
    // Extra host admission notifications cannot wrap the attempt counter.
    for _ in 0..260 {
        assert_eq!(s.picker_started(), 1);
    }
    s.retired(Role::Picker, true, context());
    assert_eq!(s.take_picker(false), Picker::Exhausted);
}
#[test]
fn selected_user_failure_resets_only_picker_budget_and_marks_error_mode() {
    let mut s = State::default();
    s.background_failed();
    assert!(s.take_background(false));
    for _ in 0..3 {
        s.picker_started();
        s.picker_started();
        assert!(s.retired(
            Role::User,
            true,
            Context {
                entered: true,
                ..context()
            }
        ));
        assert_eq!(s.take_picker(false), Picker::Restart);
        assert_eq!(s.picker_started(), 2);
        s.retired(Role::Picker, true, context());
        assert_eq!(s.take_picker(false), Picker::Restart);
        assert_eq!(s.picker_started(), 2);
        s.retired(Role::Picker, true, context());
        assert_eq!(s.take_picker(false), Picker::Exhausted);
        s.retired(
            Role::Background,
            true,
            Context {
                desktop: true,
                login: false,
                ..context()
            },
        );
        assert!(!s.take_background(false));
    }
    // Admission/placement rejection after retiring the picker uses the same reset.
    s.selection_failed(false);
    assert_eq!(s.take_picker(false), Picker::Restart);
    assert_eq!(s.picker_started(), 2);
}
#[test]
fn background_retry_is_single_use_and_independent_of_picker_work() {
    let mut s = State::default();
    s.background_failed();
    s.selection_failed(false);
    assert!(!s.take_background(true));
    assert_eq!(s.take_picker(true), Picker::Wait);
    assert_eq!(s.take_picker(false), Picker::Restart);
    assert!(s.take_background(false));
    assert!(!s.take_background(false));
    for failed in [false, true, true] {
        s.retired(
            Role::Background,
            failed,
            Context {
                login: false,
                desktop: true,
                ..context()
            },
        );
        assert!(!s.take_background(false));
    }
}

#[test]
fn owner_managed_admission_failure_never_queues_a_local_picker() {
    let mut s = State::default();
    for _ in 0..4 {
        s.selection_failed(true);
        assert_eq!(s.take_picker(false), Picker::Wait);
        assert!(!s.retired(
            Role::Picker,
            true,
            Context {
                owner_recovers: true,
                ..context()
            }
        ));
        assert_eq!(s.take_picker(false), Picker::Wait);
    }
    // It also cannot replenish a spent local budget if ownership is changed.
    s.picker_started();
    s.picker_started();
    s.selection_failed(true);
    s.retired(Role::Picker, true, context());
    assert_eq!(s.take_picker(false), Picker::Exhausted);
}
