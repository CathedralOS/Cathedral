#![no_std]
#![no_main]
//! Distribution policy configures a matrix-owned process tree through its root.
//! Role 0 is the owner: the F5 desktop with the shell feature, the probe desktop
//! with desktop alone, or a headless matrix fixture otherwise. Role 2 is a
//! supplied app, role 3 the pressure spinner, anything else a test leaf.
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
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    let _ = argument;
    match role {
        #[cfg(feature = "shell")]
        0 => session::run(),
        #[cfg(all(feature = "desktop", not(feature = "shell")))]
        0 => lab::run(),
        #[cfg(not(any(feature = "desktop", feature = "storage-lab")))]
        0 => scenario::run(),
        #[cfg(all(not(feature = "desktop"), feature = "storage-lab"))]
        0 => storage::run(),
        #[cfg(feature = "desktop")]
        2 => {
            #[cfg(feature = "shell")]
            if argument == 3 {
                return session::launcher(environment);
            }
            session::app(argument, environment)
        }
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
    let result = pair
        .0
        .send(&wire::encode(words))
        .and_then(|()| pair.1.receive_until(&mut bytes, time::after(300).unwrap()));
    let len = result.unwrap_or_else(|error| {
        let root = ROOT_TASK.load(core::sync::atomic::Ordering::Relaxed);
        panic!(
            "matrix request {words:?}: {error:?}; root {:?}",
            cathedral_user_runtime::task::Child::from_raw(root).wait_until(time::now().unwrap())
        )
    });
    wire::decode(&bytes[..len]).unwrap()
}
