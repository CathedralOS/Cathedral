#![no_std]
#![no_main]
//! Dispatch the boot-selected controller and its disposable raw-ABI peers.
mod supervisor;
mod worker;
cathedral_user_runtime::entry!(main);
fn main(role: u64, _: u64) -> u64 {
    if role == 0 {
        supervisor::run()
    } else {
        worker::run()
    }
}
