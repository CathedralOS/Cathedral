#![no_std]
#![no_main]
//! The installed Colors program has no startup-owner or device-provider code.
cathedral_user_runtime::service_imports!(&cathedral_platform_contracts::imports::REQUIRED);
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    cathedral_colors::run::<false>(0, environment, |_, _| Some(0))
}
