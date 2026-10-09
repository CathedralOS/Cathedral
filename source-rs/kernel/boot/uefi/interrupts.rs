//! Establish the exception floor before exposing the first external interrupt.

use crate::diagnostics;
use cathedral_arch as arch;
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

pub fn install(layout: &arch::BootLayout, console: &mut SerialPort) {
    // SAFETY: Sole CPU, IRQs off, owned mappings active, and permanent stacks.
    unsafe {
        arch::install_interrupts(layout, diagnostics::fault);
        arch::test_breakpoint();
    }
    writeln!(
        console,
        "Cathedral Rust lab: GDT TSS IDT installed; breakpoint returned"
    )
    .ok();
}

pub fn start_timer(layout: &arch::BootLayout, console: &mut SerialPort) {
    // SAFETY: install ran first; all handlers and the IRQ stack are resident.
    let ticks = unsafe {
        arch::start_timer();
        arch::wait_for_ticks(3)
    };
    let irq_stack = arch::last_irq_stack();
    assert!((layout.emergency_tops[3] - 16 * 1024..layout.emergency_tops[3]).contains(&irq_stack));
    writeln!(
        console,
        "Cathedral Rust lab: timer ticks={ticks} irq_rsp={irq_stack:#x}"
    )
    .ok();
}

#[cfg(not(feature = "smoke-test"))]
pub fn idle() -> ! {
    loop {
        // SAFETY: Timer/IDT are installed; idle holds no locks across interrupts.
        unsafe {
            arch::wait_for_ticks(100);
        }
    }
}
