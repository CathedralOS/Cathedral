#![no_std]
#![no_main]
//! An ordinary matrix child that lists its realm and asks its host to run,
//! copy or close what it sees. Hosted over a folder of users it is the login
//! surface; hosted inside a user's realm it is the chrome.
mod session;
cathedral_user_runtime::entry_with_environment!(main);
fn main(_: u64, _: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    session::run(environment)
}
