#![no_std]
#![no_main]
//! Distribution policy configures a matrix-owned process tree through its root.
//! Started by boot with nothing, this executable is the owner: the F5 desktop
//! with the shell feature, the probe desktop with desktop alone, or a headless
//! matrix fixture otherwise. Lab builds also host the shared Colors probe loop
//! (role 0 or 2), role 3 the pressure spinner, and test leaves. The installed
//! Colors executable lives in applications/colors and contains no startup owner.
#[cfg(not(feature = "desktop"))]
mod fixture;
#[cfg(all(feature = "desktop", not(feature = "shell")))]
mod lab;
mod leaf;
#[cfg(feature = "shell")]
#[path = "recovery/protocol.rs"]
mod recovery_protocol;
#[cfg(not(any(feature = "desktop", feature = "storage-lab")))]
mod scenario;
#[cfg(feature = "desktop")]
mod session;
#[cfg(all(not(feature = "desktop"), feature = "storage-lab"))]
mod storage;
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_kernel_contracts::user as abi;
#[cfg(feature = "desktop")]
use cathedral_user_runtime::ipc::Handle;
use cathedral_user_runtime::{Error, time};
#[cfg(feature = "desktop")]
use session::control::request as call;
#[cfg(feature = "desktop")]
type Pair = (Handle, Handle);
#[cfg(feature = "desktop")]
static ROOT_TASK: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
#[cfg(feature = "desktop")]
cathedral_user_runtime::service_imports!(&cathedral_platform_contracts::imports::REQUIRED);
#[cfg(not(feature = "desktop"))]
cathedral_user_runtime::service_bootstrap!(&cathedral_platform_contracts::imports::REQUIRED, 3, 0);
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    let _ = argument;
    let owner = role == 0 && environment.is_empty();
    match role {
        #[cfg(feature = "shell")]
        0 if owner => session::run(),
        #[cfg(all(feature = "desktop", not(feature = "shell")))]
        0 if owner => lab::run(),
        #[cfg(not(any(feature = "desktop", feature = "storage-lab")))]
        0 if owner => scenario::run(),
        #[cfg(all(not(feature = "desktop"), feature = "storage-lab"))]
        0 if owner => storage::run(),
        #[cfg(all(feature = "desktop", not(feature = "shell")))]
        0 | 2 => cathedral_colors::run::<true>(argument, environment, lab::command),
        #[cfg(feature = "desktop")]
        3 => loop {
            core::hint::spin_loop();
        },
        _ => leaf::run(environment),
    }
}
