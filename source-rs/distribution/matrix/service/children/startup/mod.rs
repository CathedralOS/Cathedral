//! Rust plumbing. Start at startup.rs.
mod lifetime;
mod picker;
pub(super) use lifetime::Recovery;
include!("startup.rs");
