//! Rust module plumbing. Start reading in `devices.rs`.

mod disk;
mod display;
pub(in crate::user_tasks) mod keyboard;
pub(in crate::user_tasks) mod leased;

include!("devices.rs");
