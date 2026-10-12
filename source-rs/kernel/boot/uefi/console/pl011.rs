//! Polling PL011 on the explicitly selected QEMU virt board, already configured by EDK2.
use core::fmt;
pub struct Console;
impl Console {
    pub fn write_bytes(&mut self, bytes: &[u8]) -> fmt::Result {
        for &byte in bytes {
            // SAFETY: QEMU virt's reserved PL011 MMIO, mapped Device by firmware and our tables.
            unsafe {
                while (0x0900_0018 as *const u32).read_volatile() & (1 << 5) != 0 {
                    core::hint::spin_loop();
                }
                (0x0900_0000 as *mut u32).write_volatile(u32::from(byte));
            }
        }
        Ok(())
    }
}
impl fmt::Write for Console {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.write_bytes(value.as_bytes())
    }
}
