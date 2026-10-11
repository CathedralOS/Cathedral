#![no_std]
#![no_main]
//! Explicit lab profiles: supervisor chooses layout; isolated clients own content.
mod client;
mod pressure;
mod scene;
mod supervisor;
use cathedral_contracts::display;
use cathedral_user_runtime::{ipc::Handle, time};
cathedral_user_runtime::entry!(main);
fn main(role: u64, _generation: u64) -> u64 {
    match role {
        0 => supervisor::run(),
        1..=3 => client::run(role),
        4 => pressure::run(),
        _ => 254,
    }
}
fn exchange(pair: (Handle, Handle), words: [u64; 6]) -> [u64; 6] {
    try_exchange(pair, words).unwrap_or_else(|error| panic!("exchange {words:?}: {error:?}"))
}
fn try_exchange(
    pair: (Handle, Handle),
    words: [u64; 6],
) -> Result<[u64; 6], cathedral_user_runtime::Error> {
    pair.0.send(&display::encode(words))?;
    let mut bytes = [0; 64];
    let len = pair.1.receive_until(&mut bytes, time::after(300)?)?;
    display::decode(&bytes[..len]).ok_or(cathedral_user_runtime::Error(
        cathedral_contracts::user::IO_ERROR as i64,
    ))
}
