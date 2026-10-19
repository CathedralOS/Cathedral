//! Rust module plumbing. Start reading in `world.rs`.

#[cfg(test)]
mod arrangements;
mod children;
mod input;
pub mod launches;
mod layout;
pub mod management;
pub mod presentation;
pub mod storage;
mod surface;
#[cfg(test)]
mod tests;
pub use children::Rect;
pub use layout::Placement;

include!("world.rs");
