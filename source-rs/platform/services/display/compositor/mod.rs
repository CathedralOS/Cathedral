//! Rust module plumbing. Start reading in `compositor.rs`.

mod configuration;
#[cfg(feature = "presentation-delay")]
mod delay_probe;
mod delivery;
mod input;
mod matrix;
mod presentation;
mod requests;
mod session;

include!("compositor.rs");
