#![no_std]
#![no_main]
mod connection;
#[cfg(feature = "recovery-lab")]
mod probes;
mod scene;
use cathedral_user_runtime::write;
cathedral_user_runtime::entry!(main);
fn main(_: u64, generation: u64) -> u64 {
    match scene::run(generation) {
        Ok(()) => 0,
        Err(_) => {
            write(b"Cathedral: application failed\n").ok();
            1
        }
    }
}
