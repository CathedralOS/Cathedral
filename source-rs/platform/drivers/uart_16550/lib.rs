#![no_std]

//! Bootstrap polling console. Runs privileged until driver isolation exists.

use core::fmt;

pub struct SerialPort {
    base: u16,
    read: unsafe fn(u16) -> u8,
    write: unsafe fn(u16, u8),
}

impl SerialPort {
    /// Initialize the QEMU PC's COM1 at 115200 baud, 8N1, interrupts disabled.
    ///
    /// # Safety
    /// Requires a 16550 at 0x3f8 and exclusive device access. The supplied port
    /// operations must implement reads/writes for that window and remain usable
    /// throughout this console's lifetime. Boot supplies privileged operations;
    /// the platform driver does not depend on the kernel implementation.
    pub unsafe fn com1(read: unsafe fn(u16) -> u8, write: unsafe fn(u16, u8)) -> Self {
        let base = 0x3f8;
        // SAFETY: The caller owns this UART's register window.
        unsafe {
            write(base + 1, 0);
            write(base + 3, 0x80);
            write(base, 1);
            write(base + 1, 0);
            write(base + 3, 3);
            write(base + 2, 0xc7);
            write(base + 4, 3);
        }
        Self { base, read, write }
    }

    fn send(&mut self, byte: u8) -> fmt::Result {
        for _ in 0..1_000_000 {
            // SAFETY: Construction establishes exclusive privileged UART access.
            if unsafe { (self.read)(self.base + 5) } & 0x20 != 0 {
                // SAFETY: Same UART ownership; transmitter reports ready.
                unsafe {
                    (self.write)(self.base, byte);
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
