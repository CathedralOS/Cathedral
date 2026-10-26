#![no_std]
#![no_main]
//! The installed Colors program has no startup-owner or device-provider code.
mod native;
cathedral_user_runtime::service_bootstrap!(&cathedral_platform_contracts::imports::REQUIRED, 3, 0);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    // SAFETY: The checked Omega export is linked into this task for its lifetime.
    unsafe { cathedral_omega_app::run(native::step(), services) }.expect("Omega app host")
}
