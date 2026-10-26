//! Rust module plumbing. Start reading in `compositor.rs`.

mod configuration;
mod delivery;
mod input;
mod matrix;
mod presentation;
mod requests;
mod session;

include!("compositor.rs");
