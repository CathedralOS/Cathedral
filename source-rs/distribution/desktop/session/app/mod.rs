//! Rust module plumbing. Start reading in `app.rs`.

#[cfg(feature = "shell")]
mod closing;
mod image;
mod state;
mod surfaces;

include!("app.rs");
