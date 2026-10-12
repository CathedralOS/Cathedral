//! Rust module plumbing. Start reading in `service.rs`.

#[cfg(feature = "recovery-lab")]
mod recovery;

include!("service.rs");
