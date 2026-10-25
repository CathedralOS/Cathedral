#![no_std]
#![no_main]
//! Capture is a supplied service; this program knows no source PID or location.
mod preview;
#[cfg(feature = "capture-probe")]
mod probe;
cathedral_user_runtime::service_bootstrap!(
    &[
        cathedral_platform_contracts::imports::Import::v1(
            cathedral_platform_contracts::imports::INTERACTION,
            true
        ),
        cathedral_platform_contracts::imports::Import::v1(
            cathedral_platform_contracts::imports::CAPTURE,
            true
        ),
    ],
    2,
    0
);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, _: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    match preview::run(services) {
        Ok(()) => 0,
        Err(error) => {
            cathedral_user_runtime::time::storage_sample(
                "capture-viewer-failed",
                &[error.0 as u64],
            );
            1
        }
    }
}
