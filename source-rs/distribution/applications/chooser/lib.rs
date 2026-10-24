#![no_std]
//! Distribution chooser decisions compiled from Omega; the guest shell starts at main.rs.
#[cfg(any(test, feature = "rust-probes"))]
pub mod policy;
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
