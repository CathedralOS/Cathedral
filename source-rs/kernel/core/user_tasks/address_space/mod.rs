mod access;
mod creation;
#[cfg(feature = "memory-lab")]
pub use creation::LoadStage;
pub use creation::{LoadBudget, LoadCursor};
mod framebuffer;
mod frames;
mod image;
mod regions;
#[cfg(feature = "memory-lab")]
pub(in crate::user_tasks) mod regions_test;
pub use image::{Contents, UserSegment, valid_segments};
include!("address_space.rs");
