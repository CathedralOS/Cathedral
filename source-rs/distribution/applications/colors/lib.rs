#![no_std]
//! Rust Colors probes retained by the mechanism labs. The installed executable
//! runs source/distribution/applications/colors through the Omega app host.
#[cfg(target_os = "none")]
mod session;
#[cfg(target_os = "none")]
pub use session::{LEFT, Probe, RIGHT, run, run_services};
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
