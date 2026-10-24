//! Rust plumbing. Start at `input.rs`.
#[cfg(test)]
mod group_tests;
mod groups;
mod keys;
mod pointer;
mod streams;
#[cfg(test)]
mod tests;
use groups::Group;
use streams::Stream;
include!("input.rs");
