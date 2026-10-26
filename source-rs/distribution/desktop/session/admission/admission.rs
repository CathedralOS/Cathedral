// Admit display, fund the root, then hand off their channels. Before commit,
// every failure retires the partial attempt. F5 adds one bounded retry before
// storage starts; catalogue labs share admission with their own proof setup.
use super::*;
use attempt::Attempt;
use cathedral_user_runtime::task::{self, Launch, Port};
#[cfg(feature = "shell")]
use cathedral_user_runtime::write;
const READY_TICKS: u64 = 300;
const BAD_REPLY: Error = Error(abi::IO_ERROR as i64);

#[derive(Clone, Copy, Debug)]
pub(super) enum Stage {
    Display = 0,
    Root = 2,
    Input = 1,
}
#[derive(Debug)]
pub(super) struct Failure {
    stage: Stage,
    error: Error,
}

/// Pre-user bootstrap: two complete attempts, never a restart loop.
#[cfg(feature = "shell")]
pub(crate) fn interactive() -> Option<Session> {
    funding::reset();
    let baseline = funding::snapshot();
    for number in 0..2 {
        write(b"Cathedral desktop: bootstrap attempt\n").unwrap();
        let result = admit(number).and_then(|mut session| {
            let result = root::configure(&mut session)
                .map_err(|error| Failure {
                    stage: Stage::Root,
                    error,
                })
                .and_then(|()| {
                    session.input(number).map_err(|error| Failure {
                        stage: Stage::Input,
                        error,
                    })
                });
            match result {
                Ok(()) => Ok(session),
                Err(error) => {
                    session.stop();
                    Err(error)
                }
            }
        });
        match result {
            Ok(session) => {
                write(b"Cathedral desktop: bootstrap accepted\n").unwrap();
                return Some(session);
            }
            Err(failure) => {
                time::storage_sample(
                    "bootstrap-failed",
                    &[number, failure.stage as u64, failure.error.0 as u64],
                );
                funding::reset();
                let reclaimed = funding::snapshot();
                assert_eq!(reclaimed, baseline, "bootstrap resource leak");
                time::storage_sample("bootstrap-reclaimed", &reclaimed);
                time::storage_sample("bootstrap-transport-reclaimed", &reclaimed[8..]);
                if number == 1 {
                    write(b"Cathedral desktop: bootstrap exhausted; restart Cathedral\n").unwrap();
                    // A failed display cannot render its own diagnostic. For
                    // root/input failure, re-admit display alone to show it.
                    if !matches!(failure.stage, Stage::Display) {
                        display::failure_screen();
                    }
                }
            }
        }
    }
    None
}
fn admit(number: u64) -> Result<Session, Failure> {
    let mut attempt = Attempt::new();
    let mut stage = Stage::Display;
    let result = (|| {
        let (display, extent) = display::start(&mut attempt, argument(stage, number, 0))?;
        labels::paint(attempt.display_control.as_mut().unwrap())?;
        stage = Stage::Root;
        let (matrix, source) = root::start(&mut attempt, display, argument(stage, number, 0))?;
        Ok((matrix, source, extent))
    })();
    match result {
        Ok((matrix, source, extent)) => Ok(attempt.commit(matrix, source, extent)),
        Err(error) => {
            attempt.retire();
            Err(Failure { stage, error })
        }
    }
}

#[cfg(not(feature = "shell"))]
pub(crate) fn start() -> Session {
    #[allow(unused_mut)]
    let mut storage = persistence::Provider::start().unwrap();
    #[cfg(feature = "notes-probes")]
    storage.arm_notes();
    #[cfg(feature = "database-probes")]
    let database_fresh = crate::lab::database_recovery::arm_startup(&mut storage);
    let mut session = admit(0)
        .unwrap_or_else(|failure| panic!("lab admission {:?}: {:?}", failure.stage, failure.error));
    session.storage = Some(storage);
    #[cfg(feature = "database-probes")]
    {
        session.database_fresh = database_fresh;
    }
    session
}

fn argument(stage: Stage, attempt: u64, normal: u64) -> u64 {
    #[cfg(feature = "bootstrap-probes")]
    return probes::argument(stage, attempt, normal);
    #[cfg(not(feature = "bootstrap-probes"))]
    {
        let _ = (stage, attempt);
        normal
    }
}
