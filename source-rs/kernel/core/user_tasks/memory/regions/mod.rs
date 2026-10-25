//! Rust module plumbing. Start reading in `regions.rs`.

mod readers;
#[cfg(test)]
mod tests;

include!("regions.rs");

#[cfg(test)]
mod shared_test;
#[cfg(test)]
mod transfer_test;
