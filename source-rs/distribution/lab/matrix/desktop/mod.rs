//! Rust module plumbing. Start reading in `desktop.rs`.

mod admission;
mod app;
mod funding;
mod probes;
mod recovery;
pub use app::run as app;

include!("desktop.rs");
