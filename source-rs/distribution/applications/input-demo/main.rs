#![no_std]
#![no_main]
//! A deliberately simple input participant. Its owner chooses its whole world.
mod participant;
#[cfg(any(feature = "hold-probe", feature = "pressure-probe"))]
mod probes;
cathedral_user_runtime::service_bootstrap!(&[
    cathedral_platform_contracts::imports::Import::v1(
        cathedral_platform_contracts::imports::INTERACTION,
        true
    ),
    cathedral_platform_contracts::imports::Import::v1(
        cathedral_platform_contracts::imports::INPUT_PRODUCER,
        true
    ),
]);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    let _ = participant::run(services);
    0
}
