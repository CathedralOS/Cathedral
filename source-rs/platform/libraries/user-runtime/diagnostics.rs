//! Bounded diagnostic writes and panic formatting; no allocator or retries.
use crate::{Error, abi, arch};

// A whole diagnostic fits this alignment, which divides a hardware page.
// Page alignment used to force several KiB of stack padding into every caller
// LLVM inlined write into, including tracing inside the compositor.
#[repr(C, align(256))]
struct AlignedWrite([u8; abi::MAX_WRITE]);
const _: () = assert!(abi::MAX_WRITE <= 256);
/// Split arbitrary slices at both ABI-size and page boundaries. One message
/// that fits a single write goes out as one call even when it straddles a
/// page in its process: split, another task's line and prefix could land
/// between its halves, and fixtures count whole lines.
pub fn write(mut bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() <= abi::MAX_WRITE && (bytes.as_ptr() as usize & 4095) + bytes.len() > 4096 {
        return crossing(bytes);
    }
    while !bytes.is_empty() {
        let length = bytes
            .len()
            .min(abi::MAX_WRITE)
            .min(4096 - (bytes.as_ptr() as usize & 4095));
        chunk(&bytes[..length])?;
        bytes = &bytes[length..];
    }
    Ok(())
}
// Keep alignment and scratch storage off the ordinary diagnostic path.
#[inline(never)]
fn crossing(bytes: &[u8]) -> Result<(), Error> {
    let mut aligned = AlignedWrite([0; abi::MAX_WRITE]);
    aligned.0[..bytes.len()].copy_from_slice(bytes);
    chunk(&aligned.0[..bytes.len()])
}
fn chunk(bytes: &[u8]) -> Result<(), Error> {
    let result = arch::call(abi::WRITE, bytes.as_ptr() as u64, bytes.len() as u64);
    if (result as i64) < 0 {
        return Err(Error(result as i64));
    }
    if result != bytes.len() as u64 {
        return Err(Error(abi::IO_ERROR as i64));
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
