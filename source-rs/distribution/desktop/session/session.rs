// The F5 desktop. Admit display/input and the funded root matrix, open storage
// behind an interactive recovery screen, then show the chooser the users realm.
// Request each selected user matrix's own startup and supervise storage.
// Admission and standing policy live in shell/; the chooser and the
// apps are programs in the store. The lab module composes the same admission,
// apps and budgets into the probe desktop from the boot catalogue.
use super::*;
use cathedral_user_runtime::{ipc, task::Launch};
pub(crate) const KEYS: u64 = if cfg!(feature = "notes") {
    wire::VALID_KEY_MASK
} else {
    wire::KEY_MASK
};
pub(crate) const BACKGROUND: u64 = 0x101827;
pub(crate) fn owner(matrix: &mut RootControl, words: wire::Packet) {
    assert_eq!(call(matrix, words), [0; 8], "owner request {words:?}");
}
#[cfg(feature = "shell")]
pub(crate) fn run() -> u64 {
    let Some(mut session) = admission::interactive() else {
        return 1;
    };
    #[cfg(any(
        feature = "launcher-read-revoke-probe",
        feature = "launcher-read-stop-probe",
        feature = "launch-read-stop-probe",
        feature = "launch-read-revoke-probe"
    ))]
    let mut read_probe = read_probe::Probe::default();
    persistence::startup::begin(&mut session);
    cathedral_user_runtime::write(b"Cathedral desktop: interactive startup ready\n").unwrap();
    #[cfg(any(
        feature = "launcher-stop-probe",
        feature = "launcher-revoke-probe",
        feature = "management-revoke-probe"
    ))]
    let mut recovery_probed = false;
    // Under storage tracing the shell samples its own account about once a
    // second of idle polling: pages and frames the kernel charges it and the
    // launch slots free, so a soak can hold the owner's account steady too.
    #[cfg(feature = "trace-storage")]
    let mut idle_polls = 0u32;
    loop {
        persistence::startup::progress(&mut session);
        // The chooser launches a user matrix into the workspace slot; the
        // desktop observes only that direct child's realm. The user matrix
        // owns its launcher, app policy and recovery bindings.
        let user = shell::realm_of(&mut session.matrix, session.user);
        if user != session.user {
            session.user = user;
            if user != 0 {
                session.login = shell::Login::Stopped;
                if let Err(code) = shell::enter(&mut session.matrix) {
                    cathedral_user_runtime::time::storage_sample("user-start-refused", &[code]);
                    shell::stop(&mut session.matrix, 1).expect("failed user cleanup");
                    session.user = 0;
                    session
                        .login
                        .start(&mut session.matrix, session.extent, true);
                    continue;
                }
                cathedral_user_runtime::write(b"Cathedral desktop: user matrix furnished\n")
                    .unwrap();
            } else {
                session
                    .login
                    .start(&mut session.matrix, session.extent, false);
                cathedral_user_runtime::write(b"Cathedral desktop: user matrix closed\n").unwrap();
            }
        }
        if session.user == 0 && session.login.poll(&mut session.matrix, session.extent) {
            if session.storage.is_none() {
                persistence::startup::begin(&mut session);
            } else {
                session
                    .login
                    .start(&mut session.matrix, session.extent, false);
            }
        }
        persistence::supervise(&mut session);
        if session.user == 0
            && session.storage_watch.offline
            && session.login == shell::Login::Picker
        {
            session
                .login
                .recover(&mut session.matrix, session.extent, abi::PEER_CLOSED);
        }
        #[cfg(any(
            feature = "launcher-stop-probe",
            feature = "launcher-revoke-probe",
            feature = "management-revoke-probe"
        ))]
        if session.user != 0 && !recovery_probed {
            recovery_probed = recovery_probes::apply(&mut session.matrix);
        }
        #[cfg(any(
            feature = "launcher-read-revoke-probe",
            feature = "launcher-read-stop-probe",
            feature = "launch-read-stop-probe",
            feature = "launch-read-revoke-probe"
        ))]
        if session.user != 0 {
            read_probe.apply(&mut session.matrix, session.extent[1]);
        }
        // Polling is only how the owner notices; rendering and input wake the
        // programs independently through their matrix connections.
        match session.matrix.idle(time::after(10).unwrap()) {
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                #[cfg(feature = "trace-storage")]
                {
                    idle_polls += 1;
                    if idle_polls.is_multiple_of(10)
                        && let Ok(budget) = cathedral_user_runtime::resources::budget()
                        && let Ok(physical) = cathedral_user_runtime::resources::physical_budget()
                    {
                        cathedral_user_runtime::time::storage_sample(
                            "shell-account",
                            &[
                                budget.used.private_pages,
                                budget.used.reader_pages,
                                physical.charged,
                                cathedral_user_runtime::task::available_slots().unwrap_or(0),
                            ],
                        );
                    }
                }
            }
            result => panic!(
                "desktop idle {result:?}; root {:?}",
                session.root.wait_until(time::now().unwrap())
            ),
        }
    }
}
