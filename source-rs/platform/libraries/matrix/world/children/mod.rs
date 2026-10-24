//! Rust plumbing. Start at children.rs for binding, delivery and lifetime.
mod directory;
#[cfg(test)]
mod directory_tests;
include!("children.rs");
