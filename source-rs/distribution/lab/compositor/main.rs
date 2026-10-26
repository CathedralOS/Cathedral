#![no_std]
#![no_main]
//! Explicit lab profiles: supervisor chooses layout; isolated clients own content.
mod client;
mod pressure;
mod scene;
mod supervisor;
mod transport;
use transport::{exchange, try_exchange};
cathedral_user_runtime::entry!(main);
fn main(role: u64, _generation: u64) -> u64 {
    match role {
        0 => supervisor::run(),
        1..=3 => client::run(role),
        4 => pressure::run(),
        _ => 254,
    }
}
