#[cfg(target_arch = "x86_64")]
pub(crate) mod pc;
#[cfg(target_arch = "aarch64")]
pub(crate) mod virt;
#[cfg(target_arch = "aarch64")]
pub use virt::Console;
mod registers;
pub use registers::DeviceId;
include!("board.rs");

#[cfg(target_arch = "x86_64")]
pub use pc::qemu_exit;
#[cfg(target_arch = "x86_64")]
pub use pc::{DiagnosticUart, diagnostic_uart};
#[cfg(target_arch = "aarch64")]
pub use virt::qemu_exit;
