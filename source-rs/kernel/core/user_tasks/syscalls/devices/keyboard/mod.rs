//! Rust module plumbing. Start reading in `keyboard.rs`.

pub(in crate::user_tasks) mod queue;

mod virtual_pointer;
pub(in crate::user_tasks) use virtual_pointer::virtual_pointer;
include!("keyboard.rs");
