//! Rust plumbing. Start at `recovery.rs`; `authority.rs` binds queued actions to live grants.
mod authority;
include!("recovery.rs");
