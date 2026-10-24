#![no_std]
//! Bounded scratchpad document; guest orchestration begins at main.rs.
pub mod document;

#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
