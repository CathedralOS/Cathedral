//! Rust module plumbing. Start reading in `children.rs`.

mod admission;
mod configuration;
mod connections;
mod control;
mod imports;
mod lifecycle;
mod loss;
mod recovery;

include!("children.rs");
