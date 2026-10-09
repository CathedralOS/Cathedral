#![no_std]
#![no_main]

//! Boot orchestration. Subsystems own firmware, memory, CPU setup and diagnostics.

extern crate alloc;

mod diagnostics;
mod firmware;
mod graphics;
mod handoff;
mod heap;
mod interrupts;
#[cfg(feature = "smoke-test")]
mod lab;
mod memory;
#[cfg(feature = "smoke-test")]
mod smoke;
#[cfg(all(feature = "bundled-user", not(feature = "smoke-test")))]
mod startup;

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
    boot.memory.frames.enable_reclamation();
    #[cfg(feature = "smoke-test")]
    lab::run(&mut boot);
    diagnostics::ready(&mut boot.console);

    #[cfg(feature = "smoke-test")]
    smoke::complete();
    #[cfg(not(feature = "smoke-test"))]
    {
        #[cfg(feature = "bundled-user")]
        startup::run(&mut boot.memory, &mut boot.console);
        interrupts::idle()
    }
}
