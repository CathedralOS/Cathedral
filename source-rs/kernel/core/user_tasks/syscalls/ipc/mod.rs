//! Rust module plumbing. Start reading in `ipc.rs`.

mod capabilities;
mod delivery;
pub(in crate::user_tasks) use delivery::Receive;
mod links;
pub(in crate::user_tasks) mod readiness;

include!("ipc.rs");
