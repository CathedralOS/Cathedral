//! QEMU-only assertions and intentional fault injection. Absent in normal boots.

use cathedral_arch as arch;
use cathedral_uart_16550::SerialPort;
use core::{
    fmt::Write,
    sync::atomic::{AtomicU64, Ordering},
};

static EXPECTED_VECTOR: AtomicU64 = AtomicU64::new(u64::MAX);
static EXPECTED_ADDRESS: AtomicU64 = AtomicU64::new(0);
static EXPECTED_STACK_TOP: AtomicU64 = AtomicU64::new(0);

pub fn probe_faults(layout: &arch::BootLayout) {
    if cfg!(feature = "fault-guard") {
        EXPECTED_VECTOR.store(14, Ordering::Release);
        EXPECTED_ADDRESS.store(layout.stack_guard, Ordering::Release);
        // SAFETY: Intentional access to an unmapped guard with diagnostics ready.
        unsafe {
            (layout.stack_guard as *mut u64).write_volatile(1);
        }
        panic!("stack guard was writable");
    } else if cfg!(feature = "fault-invalid-opcode") {
        EXPECTED_VECTOR.store(6, Ordering::Release);
        // SAFETY: Intentional invalid instruction with diagnostics ready.
        unsafe {
            arch::probe_invalid_opcode();
        }
    } else if cfg!(feature = "fault-double-fault") {
        EXPECTED_VECTOR.store(8, Ordering::Release);
        EXPECTED_STACK_TOP.store(layout.emergency_tops[0], Ordering::Release);
        // SAFETY: Deliberate stack failure, with a valid double-fault IST.
        unsafe {
            arch::probe_double_fault(layout.stack_guard);
        }
    }
}

pub fn finish_fault(fault: &arch::Fault, console: &mut SerialPort) -> ! {
    let top = EXPECTED_STACK_TOP.load(Ordering::Acquire);
    let expected = EXPECTED_VECTOR.load(Ordering::Acquire) == fault.vector
        && (fault.vector != 14
            || (fault.address == EXPECTED_ADDRESS.load(Ordering::Acquire) && fault.error & 7 == 2))
        && (fault.vector != 8
            || (fault.error == 0 && (top.saturating_sub(16 * 1024)..top).contains(&fault.stack)));
    if expected {
        writeln!(console, "CATHEDRAL_RS_EXPECTED_FAULT").ok();
        complete();
    }
    fail()
}

pub fn complete() -> ! {
    // SAFETY: Only the smoke runner supplies isa-debug-exit at port 0xf4.
    unsafe { arch::qemu_exit(0x10) }
}

pub fn fail() -> ! {
    // SAFETY: Same test-only device; this status means failure to the host.
    unsafe { arch::qemu_exit(0x11) }
}
