//! Rust module plumbing. Start reading in `timing.rs`.
mod phases;
pub(in crate::user_tasks) use phases::Phase;
#[cfg(test)]
mod tests;
#[cfg(any(feature = "memory-lab", test))]
#[deny(unsafe_code)]
mod waits;
include!("timing.rs");
