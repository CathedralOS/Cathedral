#![no_std]
#![no_main]
//! A different matrix program: three equal local views, no launcher or persistence.
//! This Rust program is the comparison path for the standalone Omega matrix in
//! source/distribution/lab/matrix-composition/process/.
mod children;
mod probes;
mod session;
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _: u64, _: cathedral_user_runtime::task::Environment) -> u64 {
    session::run().unwrap();
    0
}
