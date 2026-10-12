#![forbid(unsafe_code)]
//! Rust module plumbing. Start reading in `elf.rs`.

#[cfg(test)]
mod tests;

include!("elf.rs");
