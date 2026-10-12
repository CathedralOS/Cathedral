//! Rust module plumbing. Start reading in `input.rs`.

mod cursor;
pub mod focus;
mod queue;
mod routing;
#[cfg(test)]
mod tests;

include!("input.rs");
