//! Rust module plumbing. Start reading in `ipc.rs`.

mod budget;
mod channels;
pub use budget::{Budget, budget};
pub use channels::channel;

include!("ipc.rs");

mod memory;
