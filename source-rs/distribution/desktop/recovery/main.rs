#![no_std]
#![no_main]
//! Boot-bundled sign-in recovery. Its owner interprets the retry request;
//! this program has local drawing/input only and never repairs stored data.
mod recovery;
mod screen;
use cathedral_platform_contracts::imports::{INTERACTION, Import};
cathedral_user_runtime::service_imports!(&[Import::v1(INTERACTION, true)]);
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    recovery::run(environment)
}
