#![no_std]
//! Conformance entrance for the independently composed Omega matrix.
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows, feature = "omega"))]
mod tests;
