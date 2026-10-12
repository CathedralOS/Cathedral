//! Rust module plumbing. Start reading in `keyboard.rs`.

#[cfg(feature = "recovery-lab")]
mod recovery;

include!("keyboard.rs");
