//! Compiled Omega service ordering, progress priority and retained-event disposition.
use crate::policy::session::{self as f, Command as C, State};
use std::{vec, vec::Vec};
const IDLE: u64 = f::DEFERRED_EMPTY | f::CAN_POLL | f::HAS_WAKE;
fn step(s: &mut State, flags: u64, event: u64, command: C, result: bool) {
    assert_eq!(s.command(flags & s.observations(), event), Some(command));
    assert_eq!(
        s.command(flags & s.observations(), event),
        None,
        "issued work cannot be repeated"
    );
    assert!(!s.complete(C::Invalid, false));
    assert!(s.complete(command, result));
    assert!(!s.complete(command, result), "completion consumed once");
}
fn before_wait(s: &mut State, flags: u64) -> Vec<C> {
    let mut commands = Vec::new();
    for _ in 0..30 {
        let c = s.command(flags & s.observations(), 0).unwrap();
        if c == C::Wait {
            return commands;
        }
        commands.push(c);
        assert!(s.complete(c, false));
    }
    panic!("service turn must reach a wait within its bounded sequence");
}
#[test]
fn quiet_turn_finishes_inflight_work_before_launch_recovery_startup_and_wait() {
    let mut s = State::default();
    assert_eq!(
        before_wait(&mut s, IDLE),
        vec![
            C::Interaction,
            C::Refresh,
            C::Storage,
            C::Binding,
            C::Presentation,
            C::Selection,
            C::LaunchScene,
            C::FinishChildren,
            C::Flush,
            C::RestoreInput,
            C::Cursors,
            C::Prune,
            C::Launch,
            C::Recover,
            C::Startup,
            C::Poll,
        ]
    );
    assert!(s.complete(C::Wait, false));
    step(&mut s, IDLE, 0, C::Interaction, false);
}
#[test]
fn pending_publication_blocks_admission_and_loss_without_blocking_storage_progress() {
    let mut s = State::default();
    let busy = f::BUSY
        | f::READY_LAUNCH
        | f::REFRESH_READY
        | f::PUBLISHING
        | f::DEFERRED_READY
        | f::HAS_WAKE;
    assert_eq!(
        before_wait(&mut s, busy),
        vec![
            C::Interaction,
            C::Refresh,
            C::Storage,
            C::Binding,
            C::Presentation,
            C::Selection,
            C::LaunchScene,
            C::FinishChildren,
            C::Prune,
        ]
    );
    assert!(s.complete(C::Wait, true));
    step(&mut s, busy | f::EVENT_BLOCKED, 5, C::Cancel, false);
    step(&mut s, busy | f::EVENT_BLOCKED, 5, C::Defer, false);
    step(&mut s, IDLE, 0, C::Interaction, false);
}
#[test]
fn ready_deferred_request_runs_before_new_launch_recovery_or_input_poll() {
    let mut s = State::default();
    let flags = f::DEFERRED_READY | f::CAN_POLL | f::HAS_WAKE;
    let commands = before_wait(&mut s, flags);
    assert!(!commands.contains(&C::Launch));
    assert!(!commands.contains(&C::Recover));
    assert!(!commands.contains(&C::Poll));
    assert!(commands.contains(&C::Startup));
    assert!(s.complete(C::Wait, true));
    step(&mut s, flags, 5, C::QueryLoss, false);
    step(&mut s, flags, 5, C::Cancel, false);
    step(&mut s, flags, 5, C::Request, false);
}
#[test]
fn cancellation_precedes_deferral_and_handled_loss_consumes_the_event() {
    for handled in [false, true] {
        let mut s = State::default();
        before_wait(&mut s, IDLE);
        assert!(s.complete(C::Wait, true));
        step(&mut s, IDLE | f::EVENT_BLOCKED, 5, C::QueryLoss, handled);
        if !handled {
            step(&mut s, IDLE | f::EVENT_BLOCKED, 5, C::Cancel, false);
            step(&mut s, IDLE | f::EVENT_BLOCKED, 5, C::Defer, false);
        }
        step(&mut s, IDLE, 0, C::Interaction, false);
    }
}
#[test]
fn notification_input_is_not_repolled_before_the_selected_event() {
    for (kind, command) in [
        (1, C::Disconnect),
        (2, C::ProducerMemory),
        (3, C::StorageMemory),
        (4, C::ImageMemory),
        (5, C::Request),
    ] {
        for wake in [false, true] {
            let mut s = State::default();
            let flags = if wake { IDLE } else { IDLE & !f::HAS_WAKE };
            before_wait(&mut s, flags);
            assert!(s.complete(C::Wait, true));
            step(&mut s, flags, kind, C::QueryLoss, false);
            step(&mut s, flags, kind, C::Cancel, false);
            if !wake {
                step(&mut s, flags, kind, C::LegacyRefresh, false);
            }
            step(&mut s, flags, kind, command, false);
            step(&mut s, flags, 0, C::Interaction, false);
        }
    }
}
#[test]
fn completed_reads_and_retained_snapshots_are_delivered_when_their_owner_is_ready() {
    let mut s = State::default();
    let commands = before_wait(&mut s, IDLE | f::READY_LAUNCH | f::REFRESH_READY);
    assert_eq!(
        &commands[..5],
        &[
            C::Interaction,
            C::Refresh,
            C::Storage,
            C::AdmitReady,
            C::Binding
        ]
    );
    let i = commands
        .iter()
        .position(|c| *c == C::RefreshRetained)
        .unwrap();
    assert_eq!(commands[i - 1], C::LaunchScene);
    assert_eq!(commands[i + 1], C::FinishChildren);
}
#[test]
fn successful_launch_or_recovery_restarts_progress_before_waiting() {
    for target in [C::Launch, C::Recover] {
        let mut s = State::default();
        loop {
            let c = s.command(IDLE & s.observations(), 0).unwrap();
            assert_ne!(c, C::Wait);
            assert!(s.complete(c, c == target));
            if c == target {
                break;
            }
        }
        step(&mut s, IDLE, 0, C::Interaction, false);
    }
}
#[test]
fn every_fact_combination_is_bounded_and_missing_events_cannot_dispatch() {
    for flags in 0..1024 {
        before_wait(&mut State::default(), flags);
    }
    let mut s = State::default();
    before_wait(&mut s, IDLE);
    assert!(s.complete(C::Wait, true));
    step(&mut s, IDLE, 0, C::QueryLoss, false);
    step(&mut s, IDLE, 0, C::Cancel, false);
    assert_eq!(s.command(IDLE & s.observations(), 0), Some(C::Invalid));
    assert!(!s.complete(C::Invalid, false));
    assert_eq!(s.command(IDLE & s.observations(), 5), None);
}
