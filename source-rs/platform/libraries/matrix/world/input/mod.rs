//! Rust plumbing. Start at `input.rs`.
mod capacity;
#[cfg(test)]
mod capacity_tests;
#[cfg(test)]
mod description_tests;
mod descriptions;
#[cfg(test)]
mod group_tests;
mod groups;
mod keys;
mod names;
mod pointer;
mod producer;
#[cfg(test)]
mod producer_tests;
mod registration;
#[cfg(test)]
mod registration_tests;
mod streams;
#[cfg(test)]
mod tests;
use groups::Group;
pub(super) use names::Name;
pub(super) use producer::Producer;
use streams::Stream;
include!("input.rs");
