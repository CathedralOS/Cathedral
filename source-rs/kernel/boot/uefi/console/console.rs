// Acquire the selected board's bootstrap diagnostic sink after firmware handoff.
/// # Safety
/// Sole boot CPU; no concurrent console writer. Fault paths abandon the old writer.
pub unsafe fn acquire() -> Console {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: Caller owns COM1 on the selected QEMU PC board.
    unsafe {
        let uart = cathedral_arch::board::diagnostic_uart();
        Console::new(uart.base, uart.read, uart.write)
    }
    #[cfg(target_arch = "aarch64")]
    Console
}
