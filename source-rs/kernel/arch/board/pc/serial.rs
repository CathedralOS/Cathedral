//! Exclusive COM2 register aperture and IRQ3 lifecycle; COM1 remains boot diagnostics.
use crate::isa::x86::{in8, out8};
use core::sync::atomic::{AtomicBool, Ordering};
const BASE: u16 = 0x2f8;
static PENDING: AtomicBool = AtomicBool::new(false);

// All routines require sole-CPU custody with interrupts disabled.
pub(in crate::board) unsafe fn read(offset: u16) -> u8 {
    // SAFETY: Board facade checked offset < 8 and caller owns COM2.
    unsafe { in8(BASE + offset) }
}
pub(in crate::board) unsafe fn write(offset: u16, value: u8) {
    // SAFETY: Board facade checked offset < 8; UART cannot initiate DMA.
    unsafe {
        out8(BASE + offset, value);
    }
}
unsafe fn mask(masked: bool) {
    // SAFETY: Exclusive PIC access with interrupts disabled, preserve unrelated routes.
    unsafe {
        out8(
            0x21,
            if masked {
                in8(0x21) | 8
            } else {
                in8(0x21) & !8
            },
        );
    }
}
pub(in crate::board) unsafe fn reset() {
    // SAFETY: No driver can resume while custody is being reset.
    unsafe {
        mask(true);
        out8(BASE + 3, 3); // Clear DLAB before touching interrupt enable.
        out8(BASE + 1, 0);
        out8(BASE + 3, 0x80);
        out8(BASE, 1);
        out8(BASE + 1, 0);
        out8(BASE + 3, 3); // Known baud divisor/framing, not the previous owner's state.
        out8(BASE + 4, 0);
        out8(BASE + 2, 7); // Enable and clear both FIFOs.
        for _ in 0..16 {
            if in8(BASE + 5) & 1 == 0 {
                break;
            }
            in8(BASE);
        }
        in8(BASE + 2);
        in8(BASE + 6);
        out8(BASE + 7, 0);
    }
    PENDING.store(false, Ordering::Release);
}
pub(in crate::board) unsafe fn arm() {
    // SAFETY: Serialized live-grant transition; source remains the driver's responsibility.
    unsafe {
        mask(false);
    }
}
pub(crate) unsafe fn interrupt() {
    // A masked PIC can latch an edge after UART reset/retirement. Do not turn
    // that old edge into a new owner's notification or consume its live arm.
    // Reading IIR may acknowledge THRE; notifications are readiness hints and
    // the driver must inspect durable status (LSR/MSR), not depend on IIR history.
    // SAFETY: Dedicated COM2 route, sole CPU, IRQs disabled.
    if unsafe { in8(BASE + 2) } & 1 != 0 {
        return;
    }
    // SAFETY: IRQ3 entry; mask before PIC EOI so a stalled driver cannot storm.
    unsafe {
        mask(true);
    }
    PENDING.store(true, Ordering::Release);
}
pub(in crate::board) fn take_interrupt() -> bool {
    PENDING.swap(false, Ordering::AcqRel)
}
