//! Rust module plumbing. Start reading in `ipc.rs`.

mod bootstrap;
mod budget;
mod channels;
mod grants;
mod initial;
mod lifecycle;
pub mod link;
mod memory;
pub use abi::{MAX_ENDPOINTS, MAX_TASKS};
#[cfg(test)]
mod tests;

include!("ipc.rs");
