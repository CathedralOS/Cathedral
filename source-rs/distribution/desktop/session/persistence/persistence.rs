// Distribution recovery policy: detect dead/stalled providers or retired data
// streams, replace within a session budget, then explicitly restore matrix grants.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::{task::Port, write};
pub(crate) use policy::Watchdog;
fn revoke(session: &admission::Session) {
    for slot in 0..2 {
        owner(
            session.matrix,
            [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0],
        );
    }
}
fn regrant(session: &mut admission::Session) -> Result<(), Error> {
    session.storage = Launch::at(3)?.spawn(0)?;
    session.storage_control = Port::at(3)?.connect()?;
    session.storage_control.0.send(storage::HEALTH)?;
    let mut bytes = [0; 64];
    let n = session
        .storage_control
        .1
        .receive_until(&mut bytes, time::after(policy::STARTUP_TIMEOUT)?)?;
    if &bytes[..n] != b"ready" {
        return Err(Error(abi::IO_ERROR as i64));
    }
    #[cfg(feature = "shell")]
    super::shell::regrant(session.matrix, session.user);
    #[cfg(not(feature = "shell"))]
    {
        for slot in 0..2 {
            owner(
                session.matrix,
                [wire::STORAGE_GRANT, slot, slot, 0, 0, 0, 0, 0],
            );
        }
        owner(
            session.matrix,
            wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
        );
    }
    write(b"Cathedral matrix desktop: storage replaced; explicit grants restored\n")?;
    Ok(())
}
/// Called periodically by the owner; never blocks waiting for an ordinary probe.
/// A timed-out probe is never reused: kill, collect, close, then reconnect.
pub(crate) fn supervise(session: &mut admission::Session) {
    if session.storage_watch.offline {
        return;
    }
    let now = time::now().unwrap();
    let dead = match session.storage.wait_until(now) {
        Ok(_) => true,
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => false,
        Err(error) => panic!("storage supervision: {error:?}"),
    };
    let mut failed = dead;
    if !dead {
        let status = call(session.matrix, [wire::STORAGE_STATUS, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(status[0], 0);
        // A provider may resume after the matrix has already retired its stream.
        // A healthy control reply cannot make that old stream safe to reuse.
        failed = status[1] != 0;
    }
    if !failed {
        if let Some(deadline) = session.storage_watch.pending {
            let mut bytes = [0; 64];
            match session.storage_control.1.receive_until(&mut bytes, now) {
                Ok(n) if &bytes[..n] == b"ready" => session.storage_watch.healthy(now),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                    failed = time::reached(now, deadline);
                }
                _ => failed = true,
            }
        } else if time::reached(now, session.storage_watch.next_probe) {
            failed = session.storage_control.0.send(storage::HEALTH).is_err();
            session.storage_watch.pending = Some(now.wrapping_add(policy::PROBE_TIMEOUT));
        }
    }
    if !failed {
        return;
    }
    if !dead {
        session.storage.cancel().unwrap();
        session.storage.wait().unwrap();
        write(b"Cathedral matrix desktop: stalled storage cancelled and collected\n").unwrap();
    }
    revoke(session);
    let _ = session.storage_control.0.close();
    let _ = session.storage_control.1.close();
    if !session.storage_watch.allow_restart() {
        write(b"Cathedral matrix desktop: storage offline; restart budget exhausted\n").unwrap();
        return;
    }
    if let Err(error) = regrant(session) {
        time::storage_sample("storage-restart-failed", &[error.0 as u64]);
        let _ = session.storage.cancel();
        let _ = session.storage.wait();
        session.storage_watch.offline = true;
        write(b"Cathedral matrix desktop: storage offline; replacement failed\n").unwrap();
        return;
    }
    session.storage_watch.healthy(time::now().unwrap());
}
