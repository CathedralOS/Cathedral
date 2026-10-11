#![no_std]
#![no_main]
//! Select physical-key requests or the compositor's physical pointer/key source.
mod controller;
mod keyboard;
mod pointer;
cathedral_user_runtime::entry!(main);
fn main(mode: u64, generation: u64) -> u64 {
    match mode {
        cathedral_contracts::pointer::SOURCE_MODE => pointer::run(),
        _ => keyboard::run(mode, generation),
    }
}
