//! Rust module plumbing. Start reading in `session.rs`.

pub(crate) mod admission;

pub(crate) mod persistence;
#[cfg(any(
    feature = "launcher-stop-probe",
    feature = "launcher-revoke-probe",
    feature = "management-revoke-probe"
))]
mod recovery_probes;
#[cfg(feature = "shell")]
mod shell;

include!("session.rs");
