#![no_std]
#![no_main]
//! A second app owns a private two-record boot counter. session.rs owns its lifetime.
mod session;
cathedral_user_runtime::entry!(main);
fn main(_: u64, generation: u64) -> u64 {
    match session::run(generation) {
        Ok(()) => 0,
        Err(_) => {
            cathedral_user_runtime::write(b"Cathedral: counter stopped\n").ok();
            1
        }
    }
}
