//! Rust module plumbing. Start reading in `memory.rs`.

mod allocation;
mod closing;
mod objects;
mod retirement;
mod sharing;
pub(in crate::user_tasks) use closing::{Closing, step as close_step};
pub(super) mod frames;
#[deny(unsafe_code)]
pub(crate) mod regions;
#[cfg(feature = "memory-lab")]
pub(super) mod rollback;

include!("memory.rs");
