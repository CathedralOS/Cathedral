//! Rust module plumbing. Start reading in `user_tasks.rs`.

pub mod admission;
mod configuration;
mod execution;
pub(crate) mod lifecycle;
pub(crate) mod memory;
mod outcomes;
#[deny(unsafe_code)]
pub(crate) mod resources;
mod session;
mod startup;
mod syscalls;
#[deny(unsafe_code)]
pub(crate) mod time;
pub use configuration::{Config, Error, Executable, Program, Supervision};
#[cfg(feature = "memory-lab")]
pub use memory::rollback::exercise as exercise_memory_rollback;
pub use outcomes::{Exit, MemoryReport, Report};

include!("user_tasks.rs");
