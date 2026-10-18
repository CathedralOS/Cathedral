//! Rust plumbing. Start with shell.rs for desktop policy.
mod layout;
pub(super) use layout::select_user;
include!("shell.rs");
