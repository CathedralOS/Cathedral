#![no_std]
#![no_main]
//! Select the display protocol. Both paths own one boot-granted GOP aperture.
mod compositor;
mod direct;
mod surface;
cathedral_user_runtime::entry_with_environment!(main);
fn main(mode: u64, generation: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    match mode {
        cathedral_platform_contracts::display::composition::MODE => compositor::run(environment),
        _ => direct::run(mode, generation),
    }
}
