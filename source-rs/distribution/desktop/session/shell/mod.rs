//! Rust plumbing. Start with shell.rs for desktop policy.
mod layout;
mod login;
pub(super) use layout::select_user;
pub(crate) use login::Login;
include!("shell.rs");
