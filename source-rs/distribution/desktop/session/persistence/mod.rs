//! Rust plumbing; start with persistence.rs.
#[cfg(feature = "shell")]
mod explicit;
mod policy;
#[cfg(feature = "storage-live-probes")]
mod probes;
mod provider;
mod replacement;
#[cfg(feature = "shell")]
pub(crate) mod startup;
pub(crate) use provider::Provider;
include!("persistence.rs");
