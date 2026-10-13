//! Rust module plumbing. Start reading in `memory.rs`.

mod allocation;
mod objects;
mod retirement;
mod sharing;
pub(super) use retirement::close;
pub(super) mod frames;
#[deny(unsafe_code)]
pub(crate) mod regions;
#[cfg(feature = "memory-lab")]
pub(super) mod rollback;

include!("memory.rs");
