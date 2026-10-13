#![no_std]
#![no_main]
//! Ordinary A -> B -> C forwarding over dynamically created channels.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    memory::{Private, raw},
};
cathedral_user_runtime::entry!(main);
mod forwarder;
mod probes;
mod producer;
mod reader;
fn main(role: u64, mode: u64) -> u64 {
    match role {
        0 => producer::run(mode),
        1 => forwarder::run(mode),
        2 => reader::run(mode),
        _ => 255,
    }
}
fn introduced(input: Handle) -> Handle {
    input.receive_with(&mut [0; 64]).unwrap().1.unwrap()
}
