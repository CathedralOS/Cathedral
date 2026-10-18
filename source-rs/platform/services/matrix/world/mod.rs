//! Rust module plumbing. Start reading in `world.rs`.

mod children;
mod input;
mod layout;
pub mod presentation;
pub mod storage;
mod surface;
#[cfg(test)]
mod tests;
pub use children::Rect;

include!("world.rs");
