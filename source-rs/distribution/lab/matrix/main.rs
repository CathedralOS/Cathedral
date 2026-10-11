#![no_std]
#![no_main]
//! Distribution policy configures a matrix-owned process tree through its root.
#[cfg(feature = "desktop")]
mod desktop;
mod leaf;
#[cfg(not(feature = "desktop"))]
mod scenario;
use cathedral_contracts::{matrix as wire, user as abi};
use cathedral_user_runtime::{Error, ipc::Handle, time};
type Pair = (Handle, Handle);
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    let _ = argument;
    match role {
        #[cfg(feature = "desktop")]
        0 => desktop::run(),
        #[cfg(not(feature = "desktop"))]
        0 => scenario::run(),
        #[cfg(feature = "desktop")]
        2 => desktop::app(argument, environment),
        _ => leaf::run(environment),
    }
}
fn call(pair: Pair, words: wire::Packet) -> wire::Packet {
    pair.0.send(&wire::encode(words)).unwrap();
    let mut bytes = [0; 64];
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    wire::decode(&bytes[..len]).unwrap()
}
