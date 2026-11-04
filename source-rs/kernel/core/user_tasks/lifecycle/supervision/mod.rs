//! Rust module plumbing. Start reading in `supervision.rs`.

mod admission;
mod delegation;
#[cfg(test)]
mod tests;

include!("supervision.rs");
