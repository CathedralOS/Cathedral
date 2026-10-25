#![no_std]
//! Stock desktop policy; executable orchestration starts at main.rs.
pub mod policy;
pub mod transport;
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
