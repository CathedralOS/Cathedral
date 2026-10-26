//! Rust plumbing. Start at `control.rs`.
mod link;
mod owner;
pub(super) use link::Control;
include!("control.rs");
