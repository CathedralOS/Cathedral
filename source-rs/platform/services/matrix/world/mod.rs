//! Rust module plumbing. Start reading in `world.rs`.

mod children;
mod input;
pub mod presentation;
mod surface;
#[cfg(test)]
mod tests;
pub use children::Rect;

include!("world.rs");
