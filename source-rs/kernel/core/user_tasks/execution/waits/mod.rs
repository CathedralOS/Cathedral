//! Rust module plumbing. Start reading in `waits.rs`.

#[cfg(feature = "memory-lab")]
pub(in crate::user_tasks) mod burst;
mod registry;
#[cfg(test)]
mod tests;
pub(in crate::user_tasks) use registry::{Kind, Registry};

include!("waits.rs");
