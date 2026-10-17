//! Rust module plumbing. Start reading in `session.rs`.

pub(crate) mod admission;
pub(crate) mod app;
pub(crate) mod persistence;
#[cfg(feature = "shell")]
mod shell;
#[cfg(feature = "shell")]
pub(crate) mod worlds;
pub use app::run as app;
#[cfg(feature = "shell")]
pub use shell::launcher;

include!("session.rs");
