//! Rust module plumbing. Start reading in `ipc.rs`.

mod capabilities;
pub(in crate::user_tasks) mod delivery;
pub(in crate::user_tasks) use delivery::Receive;
mod links;
pub(in crate::user_tasks) mod notifications;
pub(in crate::user_tasks) mod readiness;

include!("ipc.rs");
