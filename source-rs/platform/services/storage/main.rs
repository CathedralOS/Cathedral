#![no_std]
#![no_main]
//! Storage executable entrance. service.rs serves contracts/storage.rs messages
//! using the object-store library and ata-pio driver; init owns its lifetime.
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
