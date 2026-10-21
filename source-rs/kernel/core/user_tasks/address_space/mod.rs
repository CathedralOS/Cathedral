mod access;
mod creation;
pub use creation::LoadCursor;
mod framebuffer;
mod frames;
mod image;
mod regions;
#[cfg(feature = "memory-lab")]
pub(in crate::user_tasks) mod regions_test;
pub use image::{Contents, UserSegment, valid_segments};
include!("address_space.rs");
