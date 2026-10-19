#![no_std]
#![no_main]
//! A replaceable matrix: persistent Notes, or another copy of this same matrix.
mod session;
cathedral_user_runtime::service_bootstrap!(&cathedral_platform_contracts::imports::REQUIRED, 15, 3);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    session::run(services).unwrap();
    0
}
