//! Rust module plumbing. Start reading in `pool.rs`.

mod executables;
pub use executables::{Code, Executables};
#[cfg(test)]
mod tests;

include!("pool.rs");
