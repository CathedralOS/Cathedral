//! Rust module plumbing. Start reading in `children.rs`.

mod admission;
mod configuration;
mod control;
mod imports;
mod lifecycle;
mod loss;
mod management;
mod recovery;
mod services;

include!("children.rs");
