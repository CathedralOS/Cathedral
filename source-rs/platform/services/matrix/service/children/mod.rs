//! Rust module plumbing. Start reading in `children.rs`.

mod admission;
mod connections;
mod control;
mod loss;

include!("children.rs");
