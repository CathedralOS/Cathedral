#![no_std]
#![no_main]
//! An ordinary matrix child: supplied connections, local input, private note.
mod session;
cathedral_user_runtime::service_bootstrap!(
    &cathedral_platform_contracts::imports::OPTIONAL_STORAGE
);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, style: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    #[cfg(feature = "startup-fault")]
    if !cathedral_user_runtime::task::Environment::load()
        .unwrap()
        .is_empty()
    {
        panic!("deliberate Notes startup failure");
    }
    session::run(style, services);
    0
}
