//! Bootstrap serial reporting and terminal error paths. No firmware calls.

use cathedral_arch as arch;
use cathedral_uart_16550::SerialPort;
use core::{fmt::Write, panic::PanicInfo};

pub fn initialize() -> SerialPort {
    // SAFETY: The single-CPU QEMU PC profile provides privileged COM1 access.
    let mut console = unsafe { SerialPort::com1() };
    writeln!(console, "Cathedral Rust lab: UEFI entry").ok();
    console
}

pub fn ready(console: &mut SerialPort) {
    writeln!(console, "CATHEDRAL_RS_BOOT_OK").ok();
}

pub fn fault(fault: arch::Fault) -> ! {
    // SAFETY: Terminal trap on the sole CPU; the interrupted owner never resumes.
    let mut console = unsafe { SerialPort::com1() };
    writeln!(
        console,
        "CATHEDRAL_RS_FAULT: vector={} error={:#x} rip={:#x} cr2={:#x} rsp={:#x}",
        fault.vector, fault.error, fault.instruction, fault.address, fault.stack
    )
    .ok();
    #[cfg(feature = "smoke-test")]
    crate::smoke::finish_fault(&fault, &mut console);
    #[cfg(not(feature = "smoke-test"))]
    // SAFETY: Fatal exception cannot resume its interrupted context.
    unsafe {
        arch::halt_forever()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    // SAFETY: No unwinding/resumption; the previous UART owner is abandoned.
    unsafe {
        arch::disable_interrupts();
        let mut console = SerialPort::com1();
        writeln!(console, "CATHEDRAL_RS_PANIC: {info}").ok();
        #[cfg(feature = "smoke-test")]
        crate::smoke::fail();
        #[cfg(not(feature = "smoke-test"))]
        arch::halt_forever()
    }
}
