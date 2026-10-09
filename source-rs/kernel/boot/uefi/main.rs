#![no_std]
#![no_main]

//! Boot orchestration. Subsystems own firmware, memory, CPU setup and diagnostics.

extern crate alloc;

#[cfg(feature = "bundled-user")]
mod applications;
mod diagnostics;
mod firmware;
mod handoff;
mod heap;
mod interrupts;
#[cfg(feature = "bundled-user")]
mod ipc;
mod memory;
#[cfg(feature = "smoke-test")]
mod smoke;
mod task_lifecycle;
mod tasks;
mod users;

use uefi::{Status, entry};

#[entry]
fn main() -> Status {
    let mut console = diagnostics::initialize();
    let inventory = firmware::exit_boot_services(&mut console);
    let memory = memory::prepare(inventory, &mut console);
    handoff::enter(memory, console, kernel_main)
}

fn kernel_main(mut boot: handoff::BootState) -> ! {
    memory::confirm_handoff(&mut boot.memory, &mut boot.console);
    interrupts::install(&boot.memory.layout, &mut boot.console);
    heap::initialize(&boot.memory.layout, &mut boot.console);
    #[cfg(feature = "smoke-test")]
    heap::verify(&boot.memory.layout, &mut boot.console);
    #[cfg(feature = "smoke-test")]
    smoke::probe_faults(&boot.memory.layout);
    interrupts::start_timer(&boot.memory.layout, &mut boot.console);
    tasks::exercise(&mut boot.memory, &mut boot.console);
    task_lifecycle::exercise(&mut boot.memory, &mut boot.console);
    users::exercise(&mut boot.memory, &mut boot.console);
    #[cfg(feature = "bundled-user")]
    applications::exercise(&mut boot.memory, &mut boot.console);
    #[cfg(feature = "bundled-user")]
    ipc::exercise(&mut boot.memory, &mut boot.console);
    diagnostics::ready(&mut boot.console);

    #[cfg(feature = "smoke-test")]
    smoke::complete();
    #[cfg(not(feature = "smoke-test"))]
    interrupts::idle()
}
