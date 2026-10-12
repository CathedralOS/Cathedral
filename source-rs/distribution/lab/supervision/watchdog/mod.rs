//! Rust module plumbing. Start reading in `watchdog.rs`.

pub mod probes;
pub mod supervisor;

include!("watchdog.rs");
