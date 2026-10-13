//! Rust module plumbing. Start reading in `memory.rs`.

mod object;
pub mod raw;
pub use object::Object;

include!("memory.rs");
