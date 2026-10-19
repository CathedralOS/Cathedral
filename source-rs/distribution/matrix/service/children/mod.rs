//! Rust module plumbing. Start reading in `children.rs`.

mod admission;
mod configuration;
mod control;
mod imports;
mod launch;
mod lifecycle;
mod loss;
mod management;
mod recovery;
mod services;
mod startup;
mod workspace;

include!("children.rs");
