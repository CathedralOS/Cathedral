#![no_std]
#![no_main]
#[cfg(feature = "recovery-lab")]
mod recovery;
mod service;
cathedral_user_runtime::entry!(main);
fn main(_: u64, _: u64) -> u64 {
    match service::run() {
        Ok(()) => 0,
        Err(_) => {
            cathedral_user_runtime::write(b"Cathedral: storage failed\n").ok();
            1
        }
    }
}
