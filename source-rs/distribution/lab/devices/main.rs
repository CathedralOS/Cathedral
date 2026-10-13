#![no_std]
#![no_main]
//! Supervisor/driver executable seam: only the child receives the register grant.
mod driver;
mod supervisor;
cathedral_user_runtime::entry!(main);
fn main(role: u64, argument: u64) -> u64 {
    match role {
        0 => supervisor::run(),
        1 => driver::run(argument),
        _ => 255,
    }
}
