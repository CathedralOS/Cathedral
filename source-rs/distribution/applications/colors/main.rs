#![no_std]
#![no_main]
//! The installed Colors program has no startup-owner or device-provider code.
cathedral_user_runtime::service_bootstrap!(&cathedral_platform_contracts::imports::REQUIRED);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    cathedral_colors::run_services::<false>(0, services, |_, _| Some(0))
}
