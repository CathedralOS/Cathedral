mod exit;
pub use exit::qemu_exit;
mod console;
pub(crate) mod gic;
pub use console::Console;
include!("virt.rs");
