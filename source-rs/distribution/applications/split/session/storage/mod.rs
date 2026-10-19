//! Rust plumbing. Read `storage.rs` for forwarding, `realms.rs` for placement policy.
mod realms;
pub use realms::open;
include!("storage.rs");
