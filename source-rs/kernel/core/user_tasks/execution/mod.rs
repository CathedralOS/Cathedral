//! Rust module plumbing. Start reading in `execution.rs`.

mod traps;
pub(in crate::user_tasks) mod waits;

include!("execution.rs");
