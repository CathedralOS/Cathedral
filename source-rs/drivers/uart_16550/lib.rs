#![no_std]

//! Bootstrap polling console. Runs privileged until driver isolation exists.

use cathedral_arch::{in8, out8};
use core::fmt;

pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    /// Initialize the QEMU PC's COM1 at 115200 baud, 8N1, interrupts disabled.
    ///
    /// # Safety
    /// Requires I/O privilege, a 16550 at 0x3f8, and exclusive device access.
    pub unsafe fn com1() -> Self {
        let base = 0x3f8;
        // SAFETY: The caller owns this UART's register window.
        unsafe {
            out8(base + 1, 0);
            out8(base + 3, 0x80);
            out8(base, 1);
            out8(base + 1, 0);
            out8(base + 3, 3);
            out8(base + 2, 0xc7);
            out8(base + 4, 3);
        }
        Self { base }
    }

    fn send(&mut self, byte: u8) -> fmt::Result {
        for _ in 0..1_000_000 {
            // SAFETY: Construction establishes exclusive privileged UART access.
            if unsafe { in8(self.base + 5) } & 0x20 != 0 {
                // SAFETY: Same UART ownership; transmitter reports ready.
                unsafe {
                    out8(self.base, byte);
                }
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(fmt::Error)
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.send(b'\r')?;
            }
            self.send(byte)?;
        }
        Ok(())
    }
}
