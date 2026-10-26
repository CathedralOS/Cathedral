#![no_std]
//! Stock desktop Omega adapters and transport. main.rs enters the native host;
//! source/distribution/matrix/service/main.omg owns its service-turn sequence.
pub mod policy;
pub mod transport;
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
