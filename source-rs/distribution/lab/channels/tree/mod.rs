//! Rust module plumbing. Start reading in `tree.rs`.

mod node;
mod probes;
pub use node::run as worker;

include!("tree.rs");
