#![no_std]
#![no_main]
//! Explicit lab profile: supervisor chooses layout; two isolated clients own content.
mod client;
mod scene;
mod supervisor;
use cathedral_contracts::display;
use cathedral_user_runtime::{ipc::Handle, time};
cathedral_user_runtime::entry!(main);
fn main(role: u64, _generation: u64) -> u64 {
    match role {
        0 => supervisor::run(),
        1 | 2 => client::run(role),
        _ => 254,
    }
}
fn exchange(pair: (Handle, Handle), words: [u64; 6]) -> [u64; 6] {
    pair.0.send(&display::encode(words)).unwrap();
    let mut bytes = [0; 64];
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    display::decode(&bytes[..len]).unwrap()
}
