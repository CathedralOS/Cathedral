#[cfg(target_arch = "x86_64")]
pub(crate) mod pc;
#[cfg(target_arch = "aarch64")]
pub(crate) mod virt;
#[cfg(target_arch = "aarch64")]
pub use virt::Console;
include!("board.rs");
