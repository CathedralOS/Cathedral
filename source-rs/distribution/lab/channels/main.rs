#![no_std]
#![no_main]
//! Ordinary parent and worker: only the introduction is supplied by boot.
mod probes;
mod scenario;
mod worker;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    time,
};
cathedral_user_runtime::entry!(main);
fn main(role: u64, mode: u64) -> u64 {
    match role {
        0 => scenario::run(),
        _ => worker::run(mode),
    }
}
fn receive(input: Handle) -> ([u8; 64], Option<Handle>) {
    ipc::wait_many(&[input], time::after(300).unwrap()).unwrap();
    let mut bytes = [0; 64];
    let (_, endpoint) = input.receive_with(&mut bytes).unwrap();
    (bytes, endpoint)
}
