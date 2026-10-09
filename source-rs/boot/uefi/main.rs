#![no_std]
#![no_main]

//! Boot orchestration. Subsystems own firmware, memory, CPU setup and diagnostics.

mod diagnostics;
mod firmware;
mod handoff;
mod interrupts;
mod memory;
#[cfg(feature = "smoke-test")]
mod smoke;

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
    #[cfg(feature = "smoke-test")]
    smoke::probe_faults(&boot.memory.layout);
    interrupts::start_timer(&boot.memory.layout, &mut boot.console);
    diagnostics::ready(&mut boot.console);

    #[cfg(feature = "smoke-test")]
    smoke::complete();
    #[cfg(not(feature = "smoke-test"))]
    interrupts::idle()
}
