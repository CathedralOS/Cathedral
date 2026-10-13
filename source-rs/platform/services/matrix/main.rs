#![no_std]
#![no_main]
//! Select the supplied parent adapter, then run the same local-world service.
mod service;
cathedral_user_runtime::entry_with_environment!(main);
fn main(mode: u64, _: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    if service::run(
        environment,
        mode == cathedral_contracts::matrix::STORAGE_ROOT,
    )
    .is_ok()
    {
        0
    } else {
        1
    }
}
