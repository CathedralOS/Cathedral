//! Rust plumbing. Start in storage.rs; pending.rs forwards, binding.rs refreshes launch grants.
mod binding;
mod pending;
include!("storage.rs");
