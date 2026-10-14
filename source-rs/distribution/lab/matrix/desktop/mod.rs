//! Rust module plumbing. Start reading in `desktop.rs`.

mod admission;
mod app;
mod funding;
mod probes;
mod providers;
mod recovery;
mod stalls;
pub use app::run as app;

include!("desktop.rs");
