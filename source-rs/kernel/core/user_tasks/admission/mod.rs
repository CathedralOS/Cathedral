//! Rust module plumbing. Start reading in `admission.rs`.

mod construction;
pub mod elf;
mod image;
#[cfg(feature = "memory-lab")]
pub(in crate::user_tasks) mod rollback;
pub(in crate::user_tasks) use construction::{Draft, abort_step, advance, publish};
#[deny(unsafe_code)]
pub(crate) mod pool;

include!("admission.rs");
