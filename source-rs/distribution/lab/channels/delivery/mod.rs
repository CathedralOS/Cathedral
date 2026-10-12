//! Rust module plumbing. Start reading in `delivery.rs`.

pub(super) mod probes;
pub(super) mod worker;

include!("delivery.rs");
