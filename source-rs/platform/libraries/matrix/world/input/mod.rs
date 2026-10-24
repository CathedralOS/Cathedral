//! Rust plumbing. Start at `input.rs`.
mod keys;
mod pointer;
mod streams;
#[cfg(test)]
mod tests;
use streams::Stream;
include!("input.rs");
