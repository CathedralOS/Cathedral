//! Start storage behind an already interactive screen. Readiness is polled;
//! failure collects the exact attempt and waits for one explicit retry.
use super::*;
pub(crate) fn begin(session: &mut admission::Session) {
    assert!(session.storage.is_none());
    assert_eq!(
        session.user, 0,
        "startup recovery never replaces a live user"
    );
    session.login.waiting(session.matrix, session.extent);
    session.storage_watch.manual_start();
    write(b"Cathedral desktop: storage startup attempt\n").unwrap();
    match Provider::begin(probe_argument()) {
        Ok(provider) => session.storage = Some(provider),
        Err(error) => failed(session, error),
    }
}
fn probe_argument() -> u64 {
    #[cfg(any(
        feature = "storage-startup-fault",
        feature = "storage-startup-stall",
        feature = "storage-startup-bad-ready"
    ))]
    {
        use core::sync::atomic::{AtomicBool, Ordering};
        static FIRST: AtomicBool = AtomicBool::new(true);
        if FIRST.swap(false, Ordering::Relaxed) || cfg!(feature = "storage-startup-repeat") {
            return if cfg!(feature = "storage-startup-fault") {
                1
            } else if cfg!(feature = "storage-startup-stall") {
                2
            } else {
                3
            };
        }
    }
    0
}
pub(crate) fn progress(session: &mut admission::Session) {
    let Some(provider) = session.storage.as_mut() else {
        return;
    };
    if provider.starting.is_none() {
        return;
    }
    match provider.poll_ready() {
        Ok(false) => (),
        Ok(true) => {
            #[cfg(feature = "notes-probes")]
            provider.arm_notes();
            session.storage_watch.healthy(time::now().unwrap());
            write(b"Cathedral desktop: storage startup accepted\n").unwrap();
            session.login.start(session.matrix, session.extent, false);
            write(b"Cathedral desktop: ready; choose a user\n").unwrap();
        }
        Err(error) => failed(session, error),
    }
}
fn failed(session: &mut admission::Session, error: Error) {
    if let Some(provider) = session.storage.take() {
        provider.retire();
    }
    session.storage_watch.offline = true;
    time::storage_sample("storage-startup-failed", &[error.0 as u64]);
    session
        .login
        .recover(session.matrix, session.extent, error.0 as u64);
}
