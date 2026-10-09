//! QEMU PC platform bootstrap, not a universal x86 timer implementation.
//! PIC/PIT remain a temporary route until ACPI/APIC discovery and calibration.

use super::out8;

/// # Safety
/// Requires exclusive privileged access to the 8259 PICs, PIT and delay port.
/// IRQs must be off, with vector 32 installed before this routine is called.
pub unsafe fn start_100_hz() {
    // SAFETY: The caller owns the controller ports; all sources start masked.
    unsafe {
        out8(0x21, 0xff);
        out8(0xa1, 0xff);
        for (port, byte) in [
            (0x20, 0x11),
            (0xa0, 0x11),
            (0x21, 0x20),
            (0xa1, 0x28),
            (0x21, 4),
            (0xa1, 2),
            (0x21, 1),
            (0xa1, 1),
        ] {
            out8(port, byte);
            out8(0x80, 0);
        }
        out8(0x21, 0xff);
        out8(0xa1, 0xff);
        // Mode 3, low/high divisor; 1,193,182 / 11,932 is approximately 100 Hz.
        out8(0x43, 0x36);
        out8(0x40, 0x9c);
        out8(0x40, 0x2e);
        // Only IRQ0 is reachable. Slave PIC and all other master IRQs stay masked.
        out8(0x21, 0xfe);
    }
}
