//! Rust module plumbing. Start reading in `parent.rs`.

mod images;
mod presentation;
mod rendering;
pub(in crate::service) mod storage;

include!("parent.rs");
