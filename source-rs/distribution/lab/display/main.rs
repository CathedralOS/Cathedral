#![no_std]
#![no_main]
//! Distribution-owned test pattern and recovery policy.
mod client;
mod probes;
mod supervisor;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::Error;
cathedral_user_runtime::entry!(main);
fn main(role: u64, _: u64) -> u64 {
    assert_eq!(
        cathedral_user_runtime::display::mapping(),
        Err(Error(abi::DENIED as i64))
    );
    match role {
        0 => supervisor::run(),
        1 => client::run(),
        2 => probes::observer(),
        4 => probes::owner(),
        5 => probes::failed_admission(),
        6 => probes::peer(),
        3 => probes::unmapped(),
        _ => 254,
    }
}
