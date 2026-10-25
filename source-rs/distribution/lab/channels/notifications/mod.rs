//! Rust plumbing. Start at notifications.rs.
mod peer;
pub(super) use peer::worker;
include!("notifications.rs");
