#![no_std]
#![no_main]
//! Cathedral startup policy and service orchestration.
mod session;
use cathedral_user_runtime::write;
cathedral_user_runtime::entry!(main);

fn main(_: u64, _: u64) -> u64 {
    match session::run() {
        Ok(()) => 0,
        Err(_) => {
            write(b"Cathedral: startup failed\n").ok();
            1
        }
    }
}
