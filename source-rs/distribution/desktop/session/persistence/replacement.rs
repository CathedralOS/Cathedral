//! A replacement owns a fresh control stream. Only accepted readiness restores
//! grants; a refused or timed-out attempt leaves live apps and their drafts alone.
use super::*;

pub(super) fn begin(session: &mut admission::Session) {
    #[cfg(feature = "shell")]
    {
        let argument = if cfg!(feature = "storage-live-readiness-stall") {
            2
        } else {
            0
        };
        match Provider::begin(argument) {
            Ok(provider) => {
                session.storage = Some(provider);
                session.storage_watch.replacing = true;
                write(b"Cathedral desktop: storage replacement starting\n").unwrap();
            }
            Err(error) => failed(session, error),
        }
    }
    #[cfg(not(feature = "shell"))]
    match Provider::start() {
        Ok(provider) => {
            session.storage = Some(provider);
            accepted(session);
        }
        Err(error) => failed(session, error),
    }
}

#[cfg(feature = "shell")]
pub(super) fn progress(session: &mut admission::Session) {
    match session
        .storage
        .as_mut()
        .expect("pending replacement")
        .poll_ready()
    {
        Ok(false) => (),
        Ok(true) => {
            session.storage_watch.replacing = false;
            accepted(session);
        }
        Err(error) => failed(session, error),
    }
}

fn accepted(session: &mut admission::Session) {
    if let Err(error) = regrant(session) {
        failed(session, error);
        return;
    }
    session.storage_watch.healthy(time::now().unwrap());
}

fn failed(session: &mut admission::Session, error: Error) {
    time::storage_sample("storage-restart-failed", &[error.0 as u64]);
    if let Some(provider) = session.storage.take() {
        provider.retire();
    }
    #[cfg(feature = "shell")]
    {
        session.storage_watch.replacing = false;
    }
    session.storage_watch.offline = true;
    write(b"Cathedral matrix desktop: storage offline; replacement failed\n").unwrap();
}
