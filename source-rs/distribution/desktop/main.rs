#![no_std]
#![no_main]
//! Distribution policy configures a matrix-owned process tree through its root.
//! Started by boot with nothing, this executable is the owner: the F5 desktop
//! with the shell feature, the probe desktop with desktop alone, or a headless
//! matrix fixture otherwise. Started by a matrix with a bundle it is a supplied
//! app (role 0 from the store or 2 from the catalogue), role 3 the pressure
//! spinner, and anything else a test leaf.
#[cfg(not(feature = "desktop"))]
mod fixture;
#[cfg(all(feature = "desktop", not(feature = "shell")))]
mod lab;
mod leaf;
#[cfg(not(any(feature = "desktop", feature = "storage-lab")))]
mod scenario;
#[cfg(feature = "desktop")]
mod session;
#[cfg(all(not(feature = "desktop"), feature = "storage-lab"))]
mod storage;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{Error, ipc::Handle, time};
type Pair = (Handle, Handle);
#[cfg(feature = "desktop")]
static ROOT_TASK: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
cathedral_user_runtime::service_imports!(&cathedral_platform_contracts::imports::REQUIRED);
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
        #[cfg(feature = "desktop")]
        0 | 2 => session::app(argument, environment),
        #[cfg(feature = "desktop")]
        3 => loop {
            core::hint::spin_loop();
        },
        _ => leaf::run(environment),
    }
}
#[cfg(feature = "desktop")]
fn call(pair: Pair, words: wire::Packet) -> wire::Packet {
    let mut bytes = [0; 64];
    let result = pair.0.send(&wire::encode(words)).and_then(|()| {
        pair.1.receive_until(
            &mut bytes,
            time::after(if words[0] & 0xffff == wire::CONFIGURE {
                wire::CONFIGURE_DEADLINE_TICKS + 100
            } else {
                300
            })
            .unwrap(),
        )
    });
    let len = result.unwrap_or_else(|error| {
        let root = ROOT_TASK.load(core::sync::atomic::Ordering::Relaxed);
        panic!(
            "matrix request {words:?}: {error:?}; root {:?}",
            cathedral_user_runtime::task::Child::from_raw(root).wait_until(time::now().unwrap())
        )
    });
    wire::decode(&bytes[..len]).unwrap()
}
