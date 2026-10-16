#![no_std]
#![no_main]
//! CPU-portability fixture: exchange capabilities, retain private state, and contain bad accesses.
mod exchange;
mod isolation;
mod sharing;
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle};
cathedral_user_runtime::entry!(main);
fn main(role: u64, address: u64) -> u64 {
    match role {
        0 => exchange::server(),
        1 => exchange::client(),
        2 | 4 | 5 => isolation::fault(role, address),
        3 => isolation::survive(address as u8),
        6 | 8 => sharing::produce(role == 8),
        7 => sharing::consume(),
        9 => sharing::supervise(address != 0),
        _ => 254,
    }
}
