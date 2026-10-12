//! Rust module plumbing. Start reading in `admission.rs`.

pub mod elf;
#[deny(unsafe_code)]
pub(crate) mod pool;

include!("admission.rs");
