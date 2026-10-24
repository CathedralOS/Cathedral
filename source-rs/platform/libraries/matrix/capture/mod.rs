//! Rust plumbing. Start at `capture.rs`.
mod geometry;
mod raster;
#[cfg(test)]
mod tests;
pub use geometry::{Plan, Rect};
pub use raster::{Layer, Paint};
include!("capture.rs");
