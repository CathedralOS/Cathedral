//! Bounded diagnostic writes and panic formatting; no allocator or retries.
use crate::{Error, abi, arch};

/// Split arbitrary slices at both ABI-size and page boundaries.
pub fn write(mut bytes: &[u8]) -> Result<(), Error> {
    while !bytes.is_empty() {
        let address = bytes.as_ptr() as u64;
        let length = bytes
            .len()
            .min(abi::MAX_WRITE)
            .min(4096 - (address as usize & 4095));
        let result = arch::call(abi::WRITE, address, length as u64);
        if (result as i64) < 0 {
            return Err(Error(result as i64));
        }
        if result != length as u64 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        bytes = &bytes[length..];
    }
    Ok(())
}

/// Best-effort diagnostics before the existing panic exit. No allocation or retry.
pub fn report_panic(info: &core::panic::PanicInfo<'_>) {
    struct Diagnostic {
        bytes: [u8; 256],
        len: usize,
    }
    impl core::fmt::Write for Diagnostic {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let count = text.len().min(self.bytes.len() - self.len);
            self.bytes[self.len..self.len + count].copy_from_slice(&text.as_bytes()[..count]);
            self.len += count;
            if count < text.len() {
                Err(core::fmt::Error)
            } else {
                Ok(())
            }
        }
    }
    use core::fmt::Write;
    let mut output = Diagnostic {
        bytes: [0; 256],
        len: 0,
    };
    let _ = writeln!(output, "Cathedral user panic: {info}");
    let _ = write(&output.bytes[..output.len]);
}
