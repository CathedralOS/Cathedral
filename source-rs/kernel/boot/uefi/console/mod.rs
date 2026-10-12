//! Rust console wiring; start in console.rs.
#[cfg(target_arch = "x86_64")]
pub type Console = cathedral_uart_16550::SerialPort;
#[cfg(target_arch = "aarch64")]
pub use cathedral_arch::board::Console;
include!("console.rs");
