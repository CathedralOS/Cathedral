#![no_std]
#![no_main]
//! Cathedral's stock matrix program, reused at every level of its distribution.
//! Desktop configuration and lifecycle policy live here, not in platform. The
//! platform matrix library supplies the local authority machinery. The environment
//! selects the adapter: provider links make a storage root, four endpoints a display root,
//! an upstream matrix bundle a nested world. Startup intent selects picker or
//! workspace policy; it never supplies capabilities or bypasses an enclosing matrix.
mod service;
#[cfg(feature = "bootstrap-probes")]
mod startup_probe;
#[cfg(not(feature = "nested-desktop"))]
cathedral_user_runtime::service_imports!(&cathedral_platform_contracts::imports::REQUIRED);
#[cfg(feature = "nested-desktop")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".cathedral.imports")]
pub static CATHEDRAL_IMPORTS: [u8; cathedral_platform_contracts::imports::PROFILE_BYTES] =
    cathedral_platform_contracts::imports::encode_profile(
        &cathedral_platform_contracts::imports::REQUIRED,
        40,
        5,
        [320, 512, 24],
    );
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(feature = "bootstrap-probes")]
    startup_probe::run(_argument >> 56);
    let startup = if cfg!(feature = "nested-desktop") {
        service::Startup::Picker
    } else if _argument == cathedral_desktop_matrix_contracts::matrix::WORKSPACE_ARGUMENT {
        service::Startup::Workspace
    } else {
        service::Startup::Hosted
    };
    match service::run(environment, startup) {
        Ok(()) => 0,
        Err(error) => {
            cathedral_user_runtime::time::storage_sample("matrix-service-exit", &[error.0 as u64]);
            1
        }
    }
}
