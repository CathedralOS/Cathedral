#![no_std]
#![no_main]
//! Replaceable background. All presentation remains within its supplied matrix.
mod native;
use cathedral_platform_contracts::imports::{INTERACTION, Import};
cathedral_user_runtime::service_bootstrap!(&[Import::v1(INTERACTION, true)], 2, 0);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    #[cfg(feature = "startup-fault")]
    if core::hint::black_box(true) {
        panic!("deliberate background startup failure");
    }
    // SAFETY: The checked Omega export is linked into this task for its lifetime.
    unsafe { cathedral_omega_app::run(native::step(), services) }.expect("Omega background host")
}
