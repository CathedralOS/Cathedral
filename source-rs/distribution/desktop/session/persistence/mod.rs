//! Rust plumbing; start with persistence.rs.
mod policy;
mod provider;
#[cfg(feature = "shell")]
pub(crate) mod startup;
pub(crate) use provider::Provider;
include!("persistence.rs");
