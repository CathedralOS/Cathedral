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
        Err(error) => {
            report(error);
            1
        }
    }
}
fn report(error: cathedral_user_runtime::Error) {
    let mut message = *b"Cathedral: storage failed code=0000000000000000\n";
    for (index, digit) in message[30..46].iter_mut().enumerate() {
        *digit = b"0123456789abcdef"[((error.0 as u64 >> ((15 - index) * 4)) & 15) as usize];
    }
    cathedral_user_runtime::write(&message).ok();
}
