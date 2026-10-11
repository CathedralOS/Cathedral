#![no_std]
#![no_main]
//! Select the display protocol. Both paths own one boot-granted GOP aperture.
mod compositor;
mod direct;
mod surface;
cathedral_user_runtime::entry!(main);
fn main(mode: u64, generation: u64) -> u64 {
    match mode {
        cathedral_contracts::composition::MODE => compositor::run(),
        _ => direct::run(mode, generation),
    }
}
