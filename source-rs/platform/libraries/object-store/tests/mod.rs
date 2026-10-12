//! Rust module plumbing. Start reading in `tests.rs`.

mod catalog;
mod disk;
mod durability;
mod protocol;

include!("tests.rs");
