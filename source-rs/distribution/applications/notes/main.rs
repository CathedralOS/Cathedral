#![no_std]
#![no_main]
//! An ordinary matrix child: supplied connections, local input, private note.
mod session;
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, style: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    session::run(style, environment);
    0
}
