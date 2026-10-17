#![no_std]
#![no_main]
//! Cold-boot private storage proof. Supervisor starts the platform provider;
//! clients use only their local link and sealed memory, without a disk grant.
mod client;
#[cfg(feature = "retention")]
mod retention;
mod supervisor;
cathedral_user_runtime::entry!(main);
fn main(role: u64, argument: u64) -> u64 {
    if role == 0 {
        supervisor::run()
    } else {
        #[cfg(feature = "retention")]
        {
            retention::run(argument)
        }
        #[cfg(not(feature = "retention"))]
        {
            client::run(argument)
        }
    }
}
