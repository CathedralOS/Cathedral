//! One launcher action permits one provider attempt, leaving the automatic
//! restart budget consumed. Live app processes and their drafts are untouched.
use super::*;
pub(super) fn poll(session: &mut admission::Session) {
    if session.user == 0 {
        return;
    }
    let state = if session.storage_watch.offline {
        wire::SAVES_OFFLINE
    } else if session.storage_watch.replacing {
        wire::SAVES_RESTARTING
    } else {
        wire::SAVES_READY
    };
    let reply = call(
        session.matrix,
        [wire::SAVE_RECOVERY_POLL, state, 0, 0, 0, 0, 0, 0],
    );
    if reply[..2] != [0, 1] || state != wire::SAVES_OFFLINE {
        return;
    }
    #[cfg(feature = "storage-live-probes")]
    probes::disable();
    session.storage_watch.manual_start();
    write(b"Cathedral desktop: explicit save retry\n").unwrap();
    replacement::begin(session);
}
