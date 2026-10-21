//! Rust module plumbing. Start reading in `startup.rs`.

mod arena;
mod composition;
#[cfg(test)]
mod device_test;

include!("startup.rs");
