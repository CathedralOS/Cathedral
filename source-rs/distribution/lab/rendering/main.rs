#![no_std]
#![no_main]
//! Guest entry: a supervisor and two peers render through real private/shared mappings.
mod guest;
cathedral_user_runtime::entry!(main);
fn main(role: u64, _: u64) -> u64 {
    match role {
        0 => guest::supervise(),
        1 => guest::produce(),
        2 => guest::compose(),
        3 => guest::bounds::guard_fault(),
        _ => panic!(),
    }
}
