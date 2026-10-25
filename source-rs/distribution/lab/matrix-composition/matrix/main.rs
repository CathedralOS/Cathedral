#![no_std]
#![no_main]
//! A different matrix program: three equal local views, no launcher or persistence.
mod children;
#[cfg(feature = "omega")]
mod host;
mod probes;
#[cfg(not(feature = "omega"))]
mod session;
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _: u64, _environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(not(feature = "omega"))]
    session::run().unwrap();
    #[cfg(feature = "omega")]
    host::run(_environment).unwrap();
    0
}
