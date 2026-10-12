#![no_std]
#![no_main]
//! Ordinary parent and worker: only the introduction is supplied by boot.
mod admission;
mod probes;
mod resources;
mod scenario;
mod sponsorship;
mod startup;
mod tree;
mod worker;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    time,
};
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, mode: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    if cfg!(feature = "dynamic-admission") && role == 0 {
        return admission::run();
    }
    if role != 0 && (30..=35).contains(&mode) {
        return admission::worker(role, mode);
    }
    if role != 0 && (20..=21).contains(&mode) {
        return resources::worker(mode);
    }
    if role == 2 {
        return tree::worker(mode, environment);
    }
    if role != 0 && (3..=6).contains(&mode) {
        return startup::worker(mode, environment);
    }
    assert!(environment.is_empty());
    if role != 0 && (7..=8).contains(&mode) {
        return sponsorship::worker(mode);
    }
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
