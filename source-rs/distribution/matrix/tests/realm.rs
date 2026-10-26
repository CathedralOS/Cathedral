//! Compiled Omega realm sequencing, cleanup obligations and deferred discovery.
use crate::policy::realm::{Command as C, Job, State};
use cathedral_kernel_contracts::user as abi;
use std::{vec, vec::Vec};
fn step(state: &mut State, expected: C, status: u64, detail: u64) {
    assert_eq!(state.command(), Some(expected));
    assert_eq!(state.command(), None, "issued operation cannot repeat");
    let wrong = if expected == C::Refresh {
        C::Notify
    } else {
        C::Refresh
    };
    assert!(!state.complete(wrong, status, detail));
    assert!(state.complete(expected, status, detail));
    assert!(
        !state.complete(expected, status, detail),
        "completion consumed once"
    );
}
fn drain(state: &mut State, failure: Option<(C, u64)>, detail: u64) -> Vec<C> {
    let mut commands = Vec::new();
    for _ in 0..40 {
        let Some(command) = state.command() else {
            return commands;
        };
        commands.push(command);
        assert!(
            state.complete(
                command,
                failure
                    .filter(|(c, _)| *c == command)
                    .map_or(0, |(_, status)| status),
                detail
            )
        );
    }
    panic!("bounded setup must finish");
}
#[test]
fn configuration_orders_image_storage_publication_and_chrome_authority() {
    let mut state = State::new(Job::Configuration, false, false, false);
    assert_eq!(
        drain(&mut state, None, 0),
        vec![
            C::Refresh,
            C::ReadConfiguration,
            C::ValidateConfiguration,
            C::AdmitLauncher,
            C::ReleaseImage,
            C::BindLauncher,
            C::InstallWorkspace,
            C::PlaceLauncher,
            C::StartBackground,
            C::Flush,
            C::Notify,
            C::GrantChrome,
            C::RecordLauncher,
            C::Finish,
        ]
    );
    assert_eq!(state.status(), 0);
    for (configured, busy, status) in [(true, false, 0), (true, true, 0), (false, true, abi::BUSY)]
    {
        let mut state = State::new(Job::Configuration, configured, busy, false);
        step(&mut state, C::Finish, 0, 0);
        assert_eq!(state.status(), status);
        assert_eq!(state.command(), None);
    }
}
#[test]
fn launcher_storage_refusals_stop_before_return_and_placement_resets_policy_first() {
    for failed in [C::AdmitLauncher, C::BindLauncher, C::PlaceLauncher] {
        let mut state = State::new(Job::Configuration, false, false, false);
        let commands = drain(&mut state, Some((failed, abi::DENIED)), 0);
        let failure = commands.iter().position(|c| *c == failed).unwrap();
        assert_eq!(
            &commands[failure + 1..],
            if failed == C::PlaceLauncher {
                &[
                    C::ClearWorkspace,
                    C::StopLauncher,
                    C::ResetSurfaces,
                    C::Finish,
                ][..]
            } else {
                &[C::StopLauncher, C::Finish][..]
            }
        );
        assert_eq!(state.status(), abi::DENIED);
        assert!(!commands.contains(&C::GrantChrome));
        if failed == C::AdmitLauncher {
            assert!(!commands.contains(&C::ReleaseImage));
        }
    }
}
#[test]
fn optional_background_refusal_records_retry_without_failing_configuration() {
    let mut state = State::new(Job::Configuration, false, false, false);
    for command in [
        C::Refresh,
        C::ReadConfiguration,
        C::ValidateConfiguration,
        C::AdmitLauncher,
        C::ReleaseImage,
        C::BindLauncher,
        C::InstallWorkspace,
        C::PlaceLauncher,
    ] {
        step(&mut state, command, 0, 0);
    }
    step(&mut state, C::StartBackground, abi::NO_MEMORY, 0);
    assert_eq!(state.last_status(), abi::NO_MEMORY);
    assert_eq!(state.status(), 0);
    step(&mut state, C::BackgroundFailed, 0, 0);
    assert_eq!(
        drain(&mut state, None, 0),
        vec![
            C::Flush,
            C::Notify,
            C::GrantChrome,
            C::RecordLauncher,
            C::Finish
        ]
    );
    assert_eq!(state.status(), 0);
}
#[test]
fn picker_and_background_keep_distinct_admission_and_cleanup_rules() {
    let mut picker = State::new(Job::Picker, false, false, false);
    assert_eq!(
        drain(&mut picker, None, 0),
        vec![
            C::PreparePicker,
            C::ReadPicker,
            C::ValidatePicker,
            C::AdmitPicker,
            C::BindPicker,
            C::PlacePicker,
            C::InstallPicker,
            C::Notify,
            C::RecordPicker,
            C::Finish
        ]
    );
    let mut background = State::new(Job::Background, false, false, false);
    assert_eq!(
        drain(&mut background, None, 0),
        vec![
            C::ReadBackground,
            C::AdmitBackground,
            C::PlaceBackground,
            C::RecordBackground,
            C::Finish
        ]
    );
    let mut background = State::new(Job::Background, false, false, false);
    assert_eq!(
        drain(&mut background, Some((C::PlaceBackground, abi::DENIED)), 0),
        vec![
            C::ReadBackground,
            C::AdmitBackground,
            C::PlaceBackground,
            C::StopBackground,
            C::Finish
        ]
    );
    assert_eq!(background.status(), abi::DENIED);
}
#[test]
fn terminal_failures_never_publish_later_roles_or_report_success() {
    for (job, failed) in [
        (Job::Configuration, C::ReadConfiguration),
        (Job::Configuration, C::ValidateConfiguration),
        (Job::Configuration, C::Flush),
        (Job::Configuration, C::GrantChrome),
        (Job::Picker, C::ReadPicker),
        (Job::Picker, C::ValidatePicker),
        (Job::Picker, C::BindPicker),
        (Job::Picker, C::PlacePicker),
        (Job::Background, C::ReadBackground),
        (Job::Background, C::AdmitBackground),
    ] {
        let mut state = State::new(job, false, false, false);
        let commands = drain(&mut state, Some((failed, abi::IO_ERROR)), 0);
        let index = commands.iter().position(|c| *c == failed).unwrap();
        assert_eq!(&commands[index + 1..], &[C::Finish]);
        assert_eq!(state.status(), abi::IO_ERROR);
    }
    let mut state = State::new(Job::Background, false, false, false);
    step(&mut state, C::ReadBackground, 0, 0);
    step(&mut state, C::AdmitBackground, 0, 0);
    step(&mut state, C::PlaceBackground, abi::DENIED, 0);
    step(&mut state, C::StopBackground, abi::IO_ERROR, 0);
    step(&mut state, C::Finish, 0, 0);
    assert_eq!(
        state.status(),
        abi::IO_ERROR,
        "cleanup failure is not successful reclamation"
    );
}
#[test]
fn discovery_prefers_configuration_and_requires_both_picker_entries() {
    for entries in 0..8 {
        let mut state = State::new(Job::Discovery, false, false, true);
        step(&mut state, C::Refresh, 0, 1);
        step(&mut state, C::InspectRealm, 0, entries);
        let expected = if entries & 1 != 0 {
            Some(C::ConfigureRealm)
        } else if entries & 6 == 6 {
            Some(C::StartPicker)
        } else {
            None
        };
        if let Some(command) = expected {
            step(&mut state, command, 0, 0);
        }
        step(&mut state, C::MarkHosted, 0, 0);
        step(&mut state, C::RecoverStartup, 0, 0);
        step(&mut state, C::Finish, 0, 0);
        assert_eq!(state.status(), 0);
    }
}
#[test]
fn unplaced_discovery_retries_without_becoming_hosted_or_consuming_recovery() {
    let mut state = State::new(Job::Discovery, false, false, true);
    step(&mut state, C::Refresh, 0, 0);
    step(&mut state, C::Finish, 0, 0);
    assert_eq!(state.status(), 0);
    let mut state = State::new(Job::Discovery, false, false, false);
    assert_eq!(
        drain(&mut state, None, 0),
        vec![C::RecoverStartup, C::Finish]
    );
}
#[test]
fn missing_realm_authority_is_hosted_but_io_and_setup_failures_propagate() {
    for code in [abi::DENIED, abi::PEER_CLOSED, abi::IO_ERROR, abi::NO_MEMORY] {
        let mut state = State::new(Job::Discovery, false, false, true);
        step(&mut state, C::Refresh, 0, 1);
        step(&mut state, C::InspectRealm, code, 7);
        let hosted = matches!(code, abi::DENIED | abi::PEER_CLOSED);
        if hosted {
            step(&mut state, C::MarkHosted, 0, 0);
            step(&mut state, C::RecoverStartup, 0, 0);
        }
        step(&mut state, C::Finish, 0, 0);
        assert_eq!(state.status(), if hosted { 0 } else { code });
    }
    for (entries, command) in [(1, C::ConfigureRealm), (6, C::StartPicker)] {
        let mut state = State::new(Job::Discovery, false, false, true);
        step(&mut state, C::Refresh, 0, 1);
        step(&mut state, C::InspectRealm, 0, entries);
        step(&mut state, command, abi::NO_MEMORY, 0);
        step(&mut state, C::Finish, 0, 0);
        assert_eq!(state.status(), abi::NO_MEMORY);
    }
}
