#![no_std]
#![no_main]
//! An ordinary matrix child that lists its realm and asks its host to run,
//! copy or close what it sees. Hosted over a folder of users it is the login
//! surface; hosted inside a user's realm it is the chrome.
#[cfg(not(feature = "rust-probes"))]
mod native;
#[cfg(feature = "rust-probes")]
mod session;
#[cfg(feature = "rust-probes")]
cathedral_user_runtime::service_imports!(&cathedral_platform_contracts::imports::REQUIRED);
#[cfg(feature = "rust-probes")]
cathedral_user_runtime::entry_with_environment!(main);
#[cfg(feature = "rust-probes")]
fn main(_: u64, argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    use cathedral_desktop_matrix_contracts::matrix::{launch_object, launch_role};
    let role = launch_role(argument);
    session::run(
        environment,
        matches!(role, 1 | 2),
        if role == 2 { "CANNOT OPEN" } else { "" },
        launch_object(argument),
    )
}
#[cfg(not(feature = "rust-probes"))]
cathedral_user_runtime::service_bootstrap!(&cathedral_platform_contracts::imports::REQUIRED, 3, 0);
#[cfg(not(feature = "rust-probes"))]
cathedral_user_runtime::entry_with_services!(main);
#[cfg(not(feature = "rust-probes"))]
fn main(_: u64, argument: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    // SAFETY: The checked export is linked for this task's lifetime.
    unsafe { cathedral_omega_app::run_with_argument(native::step(), services, argument) }
        .expect("Omega Chooser host")
}
