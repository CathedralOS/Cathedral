//! Rust module plumbing. Start reading in `execution.rs`.

pub(in crate::user_tasks) mod deferred;
pub(in crate::user_tasks) mod timing;
mod traps;
pub(in crate::user_tasks) mod waits;

include!("execution.rs");
