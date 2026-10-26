//! Rust module plumbing. Start reading in `session.rs`.

pub(crate) mod admission;
pub(crate) mod control;
use control::RootControl;

pub(crate) mod persistence;
#[cfg(any(
    feature = "launcher-read-revoke-probe",
    feature = "launcher-read-stop-probe",
    feature = "launch-read-stop-probe",
    feature = "launch-read-revoke-probe"
))]
mod read_probe;
#[cfg(any(
    feature = "launcher-stop-probe",
    feature = "launcher-revoke-probe",
    feature = "management-revoke-probe"
))]
mod recovery_probes;
#[cfg(feature = "shell")]
mod shell;

include!("session.rs");
