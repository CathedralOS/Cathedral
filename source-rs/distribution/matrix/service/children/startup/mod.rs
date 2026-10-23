//! Rust plumbing. Start at startup.rs.
mod lifetime;
pub(super) use lifetime::Recovery;
include!("startup.rs");
