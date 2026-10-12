#![no_std]
#![no_main]
//! Select restart, failed-admission, watchdog, sibling, link and CPU-budget fixture roles.
mod cpu;
mod failures;
mod links;
mod multiple;
mod restarts;
mod watchdog;
cathedral_user_runtime::entry!(main);
fn main(role: u64, argument: u64) -> u64 {
    match role {
        0 => restarts::run(),
        1 => restarts::client::run(),
        2 => restarts::service::run(argument),
        3 => failures::failed_spawn(argument),
        4 => failures::passive_peer(),
        5 => failures::abandon_child(),
        6 => failures::blocked_child(),
        7 => argument,
        8 => failures::returned_child(),
        9 => {
            failures::abandon_child();
            // SAFETY: Intentional supervisor fault to exercise owned-child teardown.
            unsafe {
                core::arch::asm!("ud2", options(noreturn));
            }
        }
        10 => watchdog::supervisor::run(),
        11 => watchdog::client(),
        12 => watchdog::service(argument),
        13 => watchdog::observer(),
        14 => watchdog::probes::completed(),
        15 => watchdog::probes::idle(),
        16 => multiple::restart(),
        17 => multiple::echo(),
        18 => multiple::failed(),
        19 => multiple::abandon(),
        20 => multiple::keyboard_waiter(),
        21 => links::owner(),
        22 => links::provider(),
        23 => links::application(),
        24 => cpu::owner(argument),
        25 => cpu::peer(argument),
        26 => cpu::branch(argument),
        27 => cpu::worker(argument),
        28 => cpu::restarts(),
        29 => cpu::idle_refill(),
        _ => 254,
    }
}
