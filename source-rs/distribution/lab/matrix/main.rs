#![no_std]
#![no_main]
//! Boot fixture owns process admission; ordinary leaf peers know only their parent.
mod leaf;
mod scenario;
use cathedral_contracts::{matrix as wire, user as abi};
use cathedral_user_runtime::{Error, ipc::Handle, time};
type Pair = (Handle, Handle);
cathedral_user_runtime::entry!(main);
fn main(role: u64, _: u64) -> u64 {
    match role {
        0 => scenario::run(),
        _ => leaf::run(),
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
