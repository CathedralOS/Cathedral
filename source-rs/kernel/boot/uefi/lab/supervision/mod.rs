//! Rust module plumbing. Start reading in `supervision.rs`.

mod cpu;
mod links;
mod multiple;
mod watchdog;

include!("supervision.rs");
