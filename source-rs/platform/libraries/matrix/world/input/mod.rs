//! Rust plumbing. Start at `input.rs`.
#[cfg(test)]
mod group_tests;
mod groups;
mod keys;
mod names;
mod pointer;
mod registration;
#[cfg(test)]
mod registration_tests;
mod streams;
#[cfg(test)]
mod tests;
use groups::Group;
pub(super) use names::Name;
use streams::Stream;
include!("input.rs");
