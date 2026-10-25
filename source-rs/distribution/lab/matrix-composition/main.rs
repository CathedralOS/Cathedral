#![no_std]
#![no_main]
//! Boot a separate matrix and three ordinary peers; probe only their IPC boundaries.
#[cfg(feature = "upstream-proof")]
mod nested;
mod peer;
mod proof;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{ipc::Handle, time};
type Pair = (Handle, Handle);
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(feature = "upstream-proof")]
    if role == 0 {
        return nested::run();
    } else if role == 2 {
        return nested::parent::run();
    }
    if role == 0 {
        proof::run()
    } else {
        peer::run(environment, argument)
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
