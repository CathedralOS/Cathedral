// Distribution recovery policy: detect dead/stalled providers or retired data
// streams, replace within a session budget, then explicitly restore matrix grants.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::write;
pub(crate) use policy::Watchdog;
fn revoke(session: &mut admission::Session) {
    for slot in 0..2 {
        assert_eq!(
            control::request(
                &mut session.matrix,
                [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0]
            ),
            [0; 8]
        );
    }
}
fn regrant(session: &mut admission::Session) -> Result<(), Error> {
    #[cfg(feature = "shell")]
    super::shell::regrant(session);
    #[cfg(not(feature = "shell"))]
    {
        for slot in 0..2 {
            owner(
                &mut session.matrix,
                [wire::STORAGE_GRANT, slot, slot, 0, 0, 0, 0, 0],
            );
        }
        owner(
            &mut session.matrix,
            wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
        );
    }
    write(b"Cathedral matrix desktop: storage replaced; explicit grants restored\n")?;
    Ok(())
}
/// Called periodically by the owner; never blocks waiting for an ordinary probe.
/// A timed-out probe is never reused: kill, collect, close, then reconnect.
pub(crate) fn supervise(session: &mut admission::Session) {
    #[cfg(feature = "shell")]
    explicit::poll(session);
    if session.storage_watch.offline {
        return;
    }
    #[cfg(feature = "shell")]
    if session.storage_watch.replacing {
        replacement::progress(session);
        return;
    }
    #[cfg(feature = "storage-live-probes")]
    probes::observe(session);
    let Some(provider) = session.storage.as_mut() else {
        return;
    };
    if provider.starting.is_some() {
        return;
    }
    let now = time::now().unwrap();
    let dead = match provider.child.wait_until(now) {
        Ok(_) => true,
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => false,
        Err(error) => panic!("storage supervision: {error:?}"),
    };
    let mut failed = dead;
    if !dead {
        let status = control::request(
            &mut session.matrix,
            [wire::STORAGE_STATUS, 0, 0, 0, 0, 0, 0, 0],
        );
        assert_eq!(status[0], 0);
        // A provider may resume after the matrix has already retired its stream.
        // A healthy control reply cannot make that old stream safe to reuse.
        failed = status[1] != 0;
    }
    if !failed {
        if let Some(deadline) = session.storage_watch.pending {
            match provider.poll_health() {
                Ok(true) => session.storage_watch.healthy(now),
                Ok(false) => {
                    failed = time::reached(now, deadline);
                }
                _ => failed = true,
            }
        } else if time::reached(now, session.storage_watch.next_probe) {
            failed = provider.begin_health().is_err();
            session.storage_watch.pending = Some(now.wrapping_add(policy::PROBE_TIMEOUT));
        }
    }
    if !failed {
        return;
    }
    session.storage.take().unwrap().retire();
    if !dead {
        write(b"Cathedral matrix desktop: stalled storage cancelled and collected\n").unwrap();
    }
    revoke(session);
    if !session.storage_watch.allow_restart() {
        write(b"Cathedral matrix desktop: storage offline; restart budget exhausted\n").unwrap();
        return;
    }
    replacement::begin(session);
}
