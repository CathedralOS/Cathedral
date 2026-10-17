#![no_std]
#![no_main]
//! One executable at every level. What the environment supplies selects the
//! adapter: provider links make a storage root, four endpoints a display root,
//! an upstream matrix bundle a nested world. Nothing branches on depth or on
//! an argument word.
mod service;
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    if service::run(environment).is_ok() {
        0
    } else {
        1
    }
}
