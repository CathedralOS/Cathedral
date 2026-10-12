//! Rust module plumbing. Start reading in `kernel_tasks.rs`.

mod execution;
mod lifecycle;
mod session;
mod stacks;
pub use crate::scheduler::TaskId;
pub use session::RunStats;

include!("kernel_tasks.rs");
