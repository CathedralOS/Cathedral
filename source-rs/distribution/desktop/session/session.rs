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
/// The backdrop the compositor shows behind every matrix tile. The Omega root
/// proof shifts it so the background program's own fill stays distinguishable.
pub(crate) const BACKGROUND: u64 = if cfg!(feature = "omega-root") {
    0x303030
} else {
    0x101827
};
pub(crate) fn owner(matrix: &mut RootControl, words: wire::Packet) {
    assert_eq!(call(matrix, words), [0; 8], "owner request {words:?}");
}
#[cfg(feature = "shell")]
pub(crate) fn run() -> u64 {
    let Some(mut session) = admission::interactive() else {
        return 1;
    };
    if cfg!(feature = "omega-root") {
        return omega_root(&mut session);
    }
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

/// Keep an admitted Omega root alive: start the storage service as F5 does
/// behind an interactive screen, run the user template, itself the Omega
/// matrix, nested over its realm at an inset, configure it through routed
/// owner words to run its launcher as a strip and open an app slot to the
/// launcher's launches, then park while the boot harness launches Colors
/// through the launcher and drives the display, input and storage paths
/// through both matrices.
#[cfg(feature = "shell")]
fn omega_root(session: &mut admission::Session) -> u64 {
    cathedral_user_runtime::write(b"Cathedral desktop: Omega root admitted\n").unwrap();
    let mut provider = persistence::Provider::begin(0).expect("storage service start");
    let deadline = time::after(3000).unwrap();
    while !provider.poll_ready().expect("storage readiness") {
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "storage service never reported ready"
        );
        idle(session);
    }
    cathedral_user_runtime::write(b"Cathedral desktop: Omega root storage ready\n").unwrap();
    // The stock realm: the user template, object three, is the matrix program
    // over its own realm, with the template's launcher, object four, Notes and
    // Colors beneath it. The root runs the template nested at an inset, as a
    // login does, and until startup discovery arrives configures it through
    // routed owner words: its launcher as a strip over the realm and an app
    // slot open to the launcher's launches.
    const TEMPLATE: u64 = 3;
    const LAUNCHER_OBJECT: u64 = 4;
    const NESTED_SLOT: u64 = 0;
    const APP_SLOT: u64 = 1;
    const INSET: [u64; 2] = [32, 24];
    let [width, height] = session.extent;
    let nested = [width - 2 * INSET[0], height - 2 * INSET[1]];
    for words in [
        [wire::RUN_PAGES, NESTED_SLOT, 512, 1024, 0, 0, 0, 0],
        [wire::RUN_OBJECT, NESTED_SLOT, TEMPLATE, TEMPLATE, 0, 64, 4, 0],
        [wire::SURFACE_LIMIT, NESTED_SLOT, 16, 0, 0, 0, 0, 0],
        [
            wire::PLACE,
            NESTED_SLOT,
            INSET[0],
            INSET[1],
            nested[0],
            nested[1],
            KEYS,
            0,
        ],
    ] {
        owner(&mut session.matrix, words);
    }
    cathedral_user_runtime::write(b"Cathedral desktop: Omega nested matrix placed\n").unwrap();
    for words in [
        [wire::RUN_PAGES, 0, 64, 4, 0, 0, 0, 0],
        [
            wire::RUN_OBJECT,
            0,
            LAUNCHER_OBJECT,
            TEMPLATE,
            wire::launch_argument(0, LAUNCHER_OBJECT),
            3,
            0,
            0,
        ],
        [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0],
        [wire::PLACE, 0, 0, nested[1] - 96, nested[0], 96, KEYS, 0],
        [wire::LAUNCH_POLICY, APP_SLOT, 24, 8, 3, 0, 0, 4],
        [wire::LAUNCH_PLACE, APP_SLOT, 650, 100, 300, 300, KEYS, 0],
    ] {
        owner(
            &mut session.matrix,
            wire::route(words, NESTED_SLOT as usize).unwrap(),
        );
    }
    cathedral_user_runtime::write(b"Cathedral desktop: Omega nested launcher placed\n").unwrap();
    loop {
        idle(session);
    }
}
#[cfg(feature = "shell")]
fn idle(session: &mut admission::Session) {
    match session.matrix.idle(time::after(10).unwrap()) {
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
        result => panic!(
            "Omega root idle {result:?}; root {:?}",
            session.root.wait_until(time::now().unwrap())
        ),
    }
}
