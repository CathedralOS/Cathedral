#![no_std]
#![no_main]
//! Executable entry; application.rs owns the visible app workflow.
mod application;
use cathedral_user_runtime::write;
cathedral_user_runtime::entry!(main);
fn main(_: u64, generation: u64) -> u64 {
    match application::run(generation) {
        Ok(()) => 0,
        Err(_) => {
            write(b"Cathedral: application failed\n").ok();
            1
        }
    }
}
