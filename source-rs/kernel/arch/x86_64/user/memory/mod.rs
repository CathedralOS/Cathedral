//! Rust module plumbing. Start reading in `memory.rs`.

mod access;
mod creation;
mod device;
mod paging;
pub use crate::image::{UserSegment, valid_segments};
mod regions;

include!("memory.rs");
