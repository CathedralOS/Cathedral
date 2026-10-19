#![no_std]
#![no_main]
//! A different matrix program: three equal local views, no launcher or persistence.
mod children;
mod probes;
mod session;
cathedral_user_runtime::entry!(main);
fn main(_: u64, _: u64) -> u64 {
    session::run().unwrap();
    0
}
