#![no_std]
#![no_main]
cathedral_user_runtime::service_bootstrap!(
    &[cathedral_platform_contracts::imports::Import::v1(
        cathedral_platform_contracts::imports::INTERACTION,
        true,
    )],
    2,
    0
);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    cathedral_omega_ui_demo::run(services)
}
