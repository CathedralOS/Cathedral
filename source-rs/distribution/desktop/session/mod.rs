//! Rust module plumbing. Start reading in `session.rs`.

pub(crate) mod admission;
pub(crate) mod app;
pub(crate) mod persistence;
#[cfg(any(feature = "launcher-stop-probe", feature = "launcher-revoke-probe"))]
mod recovery_probes;
#[cfg(feature = "shell")]
mod shell;
pub use app::run as app;

include!("session.rs");
