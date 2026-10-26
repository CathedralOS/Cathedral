#![no_std]
#![no_main]
//! Boot-bundled sign-in and launcher recovery. Its owner interprets the retry request;
//! this program has local drawing/input only and never repairs stored data.
mod protocol;
mod recovery;
mod screen;
use cathedral_platform_contracts::imports::{INTERACTION, Import};
cathedral_user_runtime::service_bootstrap!(&[Import::v1(INTERACTION, true)], 2, 0);
cathedral_user_runtime::entry_with_services!(main);
fn main(_: u64, argument: u64, services: cathedral_user_runtime::services::Services) -> u64 {
    recovery::run(services, argument)
}
