//! Rust module plumbing. Start reading in `lifecycle.rs`.

#[cfg(feature = "memory-lab")]
pub(in crate::user_tasks) mod cancellation_test;
mod delegation;
mod grants;
mod links;
mod reclamation;
mod spawn;
mod withdrawal;
#[cfg(feature = "memory-lab")]
pub(in crate::user_tasks) mod withdrawal_test;
pub(super) use spawn::spawn;
#[deny(unsafe_code)]
pub(crate) mod supervision;

include!("lifecycle.rs");
