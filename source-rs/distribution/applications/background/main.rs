#![no_std]
#![no_main]
//! Replaceable background. All presentation remains within its supplied matrix.
mod backdrop;
use cathedral_platform_contracts::imports::{INTERACTION, Import};
cathedral_user_runtime::service_bootstrap!(&[Import::v1(INTERACTION, true)], 1, 0);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    #[cfg(feature = "startup-fault")]
    if core::hint::black_box(true) {
        panic!("deliberate background startup failure");
    }
    backdrop::run(services).unwrap();
    0
}
