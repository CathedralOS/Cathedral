//! Rust module plumbing. Start at `matrix.rs` for startup, `admission/` for spawning and `upstream/` for parent exchanges.
pub mod admission;
pub mod upstream;
include!("matrix.rs");
