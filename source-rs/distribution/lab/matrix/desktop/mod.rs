//! Rust module plumbing. Start reading in `desktop.rs`.

mod admission;
mod app;
mod funding;
mod probes;
pub use app::run as app;

include!("desktop.rs");
