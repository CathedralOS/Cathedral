#![no_std]
//! Tests invoke the same compiled Omega background used by the installed task.
#[cfg(test)]
extern crate std;
#[cfg(all(test, windows))]
mod tests;
