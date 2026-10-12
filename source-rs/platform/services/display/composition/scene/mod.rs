//! Rust module plumbing. Start reading in `scene.rs`.

mod geometry;
pub mod hosting;
pub mod labels;
mod raster;
mod snapshot;
pub mod tiles;
pub use geometry::Rect;
#[cfg(test)]
mod tests;

include!("scene.rs");
