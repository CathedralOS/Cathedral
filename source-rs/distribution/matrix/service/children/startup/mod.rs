//! Rust plumbing. Start at startup.rs.
mod lifetime;
mod realm;
pub(super) use lifetime::Recovery;
include!("startup.rs");
