//! Rust plumbing. Start in storage.rs; pending.rs forwards, binding.rs refreshes launch grants.
mod binding;
mod pending;
mod provider;
pub(super) use provider::Provider;
include!("storage.rs");
