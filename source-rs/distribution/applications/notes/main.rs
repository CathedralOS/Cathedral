#![no_std]
#![no_main]
//! An ordinary matrix child: supplied connections, local input, private note.
#[cfg(not(feature = "rust-probes"))]
mod native;
#[cfg(feature = "rust-probes")]
mod session;
#[cfg(not(feature = "input-producer-probe"))]
cathedral_user_runtime::service_bootstrap!(
    &cathedral_platform_contracts::imports::OPTIONAL_STORAGE,
    if cfg!(feature = "rust-probes") { 2 } else { 3 },
    0
);
#[cfg(feature = "input-producer-probe")]
cathedral_user_runtime::service_bootstrap!(
    &[
        cathedral_platform_contracts::imports::Import::v1(
            cathedral_platform_contracts::imports::INTERACTION,
            true
        ),
        cathedral_platform_contracts::imports::Import::v1(
            cathedral_platform_contracts::imports::STORAGE,
            false
        ),
        cathedral_platform_contracts::imports::Import::v1(
            cathedral_platform_contracts::imports::INPUT_PRODUCER,
            false
        ),
    ],
    if cfg!(feature = "rust-probes") { 2 } else { 3 },
    0
);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _style: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    #[cfg(feature = "startup-fault")]
    if !cathedral_user_runtime::task::Environment::load()
        .unwrap()
        .is_empty()
    {
        panic!("deliberate Notes startup failure");
    }
    #[cfg(feature = "rust-probes")]
    {
        session::run(_style, services);
        0
    }
    #[cfg(not(feature = "rust-probes"))]
    {
        // SAFETY: The checked export is linked for this task's lifetime.
        unsafe { cathedral_omega_app::run(native::step(), services) }.expect("Omega Notes host")
    }
}
