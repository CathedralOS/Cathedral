#![no_std]
#![no_main]
//! Ordinary parent and worker: only the introduction is supplied by boot.
mod probes;
mod scenario;
mod startup;
mod worker;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    time,
};
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, mode: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    if role != 0 && (3..=6).contains(&mode) {
        return startup::worker(mode, environment);
    }
    assert!(environment.is_empty());
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
