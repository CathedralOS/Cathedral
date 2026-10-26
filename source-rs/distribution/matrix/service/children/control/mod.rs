//! Rust plumbing. Start at `control.rs`.
mod link;
pub(super) use link::Control;
include!("control.rs");
