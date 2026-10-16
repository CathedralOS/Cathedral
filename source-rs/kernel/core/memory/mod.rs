//! Rust module plumbing. Start reading in `memory.rs`.

#[allow(unsafe_code)]
pub mod heap;
pub mod inventory;
#[cfg(test)]
mod tests;

include!("memory.rs");
