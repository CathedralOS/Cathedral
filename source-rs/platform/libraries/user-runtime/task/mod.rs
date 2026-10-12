//! Rust module plumbing. Start reading in `task.rs`.

mod environment;
mod executable;
pub use executable::{Executable, available_slots};
mod launch;
pub use environment::Environment;

include!("task.rs");
