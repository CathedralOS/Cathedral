//! Rust plumbing. Start with shell.rs for desktop policy, launcher.rs for its UI.
mod launcher;
mod view;
pub use launcher::run as launcher;
include!("shell.rs");
