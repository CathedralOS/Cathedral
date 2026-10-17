//! Rust module plumbing. Start reading in `desktop.rs`.

mod admission;
mod app;
#[cfg(feature = "database-probes")]
mod database_recovery;
mod funding;
mod persistence;
mod probes;
mod providers;
mod recovery;
mod stalls;
mod stalls_peer;
pub use app::run as app;

include!("desktop.rs");
