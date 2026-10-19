#![no_std]
#![no_main]
//! Select physical-key requests or the compositor's physical pointer/key source.
#[cfg(feature = "absolute-pointer")]
mod absolute;
mod controller;
mod keyboard;
mod matrix;
mod pointer;
#[cfg(feature = "bootstrap-probes")]
mod startup_probe;
cathedral_user_runtime::entry_with_environment!(main);
fn main(mode: u64, generation: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(feature = "bootstrap-probes")]
    startup_probe::run(generation >> 56);
    match mode {
        cathedral_platform_contracts::display::pointer::SOURCE_MODE => pointer::run(),
        cathedral_platform_contracts::matrix::SOURCE_MODE => matrix::run(generation, environment),
        _ => keyboard::run(mode, generation),
    }
}
