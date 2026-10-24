//! Rust module plumbing. Start reading in `parent.rs`.

mod cursors;
mod images;
mod input;
mod polling;
mod presentation;
mod recovery;
mod rendering;
mod scene;
pub(in crate::service) mod storage;

include!("parent.rs");
