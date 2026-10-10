//! Check providers, collect application outcomes, then answer its bounded requests.
//! Replacement decisions live here; child.rs owns cancellation/reaping/spawn mechanics.
mod control;
use super::{
    child::Service,
    providers::{self, Providers},
};
use cathedral_contracts::user as abi;
use cathedral_session_protocol as session;
use cathedral_user_runtime::{Error, time, write};
pub(super) fn run(providers: Providers, mut app: Service) -> Result<(), Error> {
    let Providers {
        mut display,
        mut input,
        mut storage,
    } = providers;
    let mut last = 0;
    let mut seen = time::now()?;
    let mut started = false;
    #[cfg(feature = "recovery-lab")]
    let mut probes = super::recovery::Probes::default();
    loop {
        if providers::display::ensure_ready(&mut display)? {
            last = 1;
        }
        if providers::input::ensure_ready(&mut input)? {
            last = 2;
        }
        if providers::storage::ensure_ready(&mut storage)? {
            last = 4;
        }
        #[cfg(feature = "recovery-lab")]
        probes.verify(&display, &input, &storage, &app)?;
        if app.stopped()? {
            let peers = (display.ticket(), input.ticket(), storage.ticket());
            app.respawn()?;
            assert_eq!((display.ticket(), input.ticket(), storage.ticket()), peers);
            last = 3;
            seen = time::now()?;
            write(b"Cathedral: application restarted; providers preserved\n")?;
        }
        let mut bytes = [0; 64];
        let length = match app.receive.receive_until(&mut bytes, time::after(10)?) {
            Ok(length) => length,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => continue,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                if time::reached(time::now()?, seen.wrapping_add(300)) {
                    let peers = (display.ticket(), input.ticket(), storage.ticket());
                    app.restart()?;
                    assert_eq!((display.ticket(), input.ticket(), storage.ticket()), peers);
                    last = 3;
                    seen = time::now()?;
                    write(b"Cathedral: application restarted; providers preserved\n")?;
                }
                continue;
            }
            Err(error) => return Err(error),
        };
        seen = time::now()?;
        let status = session::Status {
            display: display.generation(),
            input: input.generation(),
            application: app.generation(),
            last,
            storage: storage.generation(),
        };
        let encoded = status.encode();
        let response = control::response(
            &bytes[..length],
            &encoded,
            #[cfg(feature = "recovery-lab")]
            [&display, &input, &storage],
            #[cfg(feature = "recovery-lab")]
            &app,
            #[cfg(feature = "recovery-lab")]
            &mut probes,
        )?;
        // App closure or an undrained response must not take init/providers down.
        if app.send.send(response).is_err() {
            let peers = (display.ticket(), input.ticket(), storage.ticket());
            app.restart()?;
            assert_eq!((display.ticket(), input.ticket(), storage.ticket()), peers);
            last = 3;
            seen = time::now()?;
            write(b"Cathedral: application restarted; providers preserved\n")?;
        } else if &bytes[..length] == session::READY {
            write(if started {
                b"Cathedral: application ready\n"
            } else {
                b"Cathedral: startup ready\n"
            })?;
            started = true;
        }
    }
}
