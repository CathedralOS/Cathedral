//! Rust module plumbing. Start reading in `keyboard.rs`.

pub(in crate::user_tasks) mod queue;

include!("keyboard.rs");
