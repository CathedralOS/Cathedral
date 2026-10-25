#![no_std]
#![no_main]
//! Cathedral's stock matrix program, reused at every level of its distribution.
//! Desktop configuration and lifecycle policy live here, not in platform. The
//! platform matrix library supplies the local authority machinery. The environment
//! selects the adapter: provider links make a storage root, four endpoints a display root,
//! a scoped two-endpoint broker a nested world (positional bundles remain in fixtures).
//! A hosted matrix reads its realm to
//! become a workspace or a desktop; no argument or build selects that, and the
//! realm never supplies capabilities or bypasses an enclosing matrix. The
//! `desktop-host` build only declares the funding a desktop needs when it is
//! launched into an app slot, where undeclared programs get an app's share.
mod service;
#[cfg(feature = "bootstrap-probes")]
mod startup_probe;
#[cfg(not(feature = "desktop-host"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".cathedral.imports")]
pub static CATHEDRAL_IMPORTS: [u8; cathedral_platform_contracts::imports::RECORD_BYTES] =
    cathedral_platform_contracts::imports::encode_brokered(
        &cathedral_platform_contracts::imports::REQUIRED,
    );
#[cfg(feature = "desktop-host")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".cathedral.imports")]
pub static CATHEDRAL_IMPORTS: [u8; cathedral_platform_contracts::imports::PROFILE_BYTES] =
    cathedral_platform_contracts::imports::encode_profile(
        &cathedral_platform_contracts::imports::REQUIRED,
        40,
        5,
        [
            320,
            512,
            cathedral_desktop_matrix_contracts::matrix::config::HOST_SURFACES as u64,
        ],
    );
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(feature = "bootstrap-probes")]
    startup_probe::run(_argument >> 56);
    match service::run(environment) {
        Ok(()) => 0,
        Err(error) => {
            cathedral_user_runtime::time::storage_sample("matrix-service-exit", &[error.0 as u64]);
            1
        }
    }
}
