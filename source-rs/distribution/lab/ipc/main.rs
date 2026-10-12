#![no_std]
#![no_main]
//! One fixture executable, separately admitted as client/service/fault-test roles.
mod deadlines;
mod exchange;
mod lifetime;
mod probes;
mod readiness;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle};
cathedral_user_runtime::entry!(main);

fn main(role: u64, stale: u64) -> u64 {
    match role {
        0 => exchange::server(),
        1 => exchange::client(),
        2 | 5 => lifetime::receiver(role),
        3 | 4 | 6 => lifetime::owner(role),
        7 => probes::sender(),
        8 => probes::receiver(),
        9 => deadlines::receiver(stale),
        10 => deadlines::sender(stale),
        11 => deadlines::denied(),
        12 => readiness::receiver(),
        13 => readiness::sender(),
        _ => 254,
    }
}
