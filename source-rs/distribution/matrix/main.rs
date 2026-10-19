#![no_std]
#![no_main]
//! Cathedral's stock matrix program, reused at every level of its distribution.
//! Desktop configuration and lifecycle policy live here, not in platform. The
//! platform matrix library supplies the local authority machinery. The environment
//! selects the adapter: provider links make a storage root, four endpoints a display root,
//! an upstream matrix bundle a nested world. Nothing branches on depth or on
//! an argument word.
mod service;
#[cfg(feature = "bootstrap-probes")]
mod startup_probe;
cathedral_user_runtime::service_imports!(&cathedral_platform_contracts::imports::REQUIRED);
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(feature = "bootstrap-probes")]
    startup_probe::run(_argument >> 56);
    if service::run(environment).is_ok() {
        0
    } else {
        1
    }
}
