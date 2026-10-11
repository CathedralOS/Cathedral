#![no_std]
#![no_main]
//! Select the supplied parent adapter, then run the same local-world service.
mod service;
cathedral_user_runtime::entry!(main);
fn main(upstream: u64, _: u64) -> u64 {
    if service::run(upstream == 1).is_ok() {
        0
    } else {
        1
    }
}
