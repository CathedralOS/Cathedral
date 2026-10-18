//! Rust module plumbing. Start reading in `world.rs`.

mod children;
mod input;
pub mod launches;
mod layout;
pub mod presentation;
pub mod storage;
mod surface;
#[cfg(test)]
mod tests;
pub use children::Rect;
pub use layout::LauncherPlacement;

include!("world.rs");
