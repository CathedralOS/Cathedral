//! Rust module plumbing. Start reading in `syscalls.rs`.

mod admission;
pub(in crate::user_tasks) mod devices;
mod diagnostics;
pub(super) mod ipc;
mod memory;
mod resources;
pub(super) mod tasks;
mod time;
pub(super) use ipc::{Receive, wake_receivers};

include!("syscalls.rs");
