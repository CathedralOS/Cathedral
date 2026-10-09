#![no_std]
#![no_main]
//! Distribution policy: restart a deliberately crashing service 32 times.
mod client;
mod probes;
mod service;
mod supervisor;
cathedral_user_runtime::entry!(main);
fn main(role: u64, argument: u64) -> u64 {
    match role {
        0 => supervisor::run(),
        1 => client::run(),
        2 => service::run(argument),
        3 => probes::failed_spawn(argument),
        4 => probes::passive_peer(),
        5 => probes::abandon_child(),
        6 => probes::blocked_child(),
        7 => argument,
        8 => probes::returned_child(),
        9 => {
            probes::abandon_child();
            // SAFETY: Intentional supervisor fault to exercise owned-child teardown.
            unsafe {
                core::arch::asm!("ud2", options(noreturn));
            }
        }
        _ => 254,
    }
}
