//! Rust module plumbing. Start reading in `waits.rs`.

mod registry;
#[cfg(test)]
mod tests;
pub(in crate::user_tasks) use registry::{Kind, Registry};

include!("waits.rs");
