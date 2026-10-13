mod exit;
pub use exit::qemu_exit;
pub(super) mod disk;
pub(super) mod keyboard;
pub(crate) mod serial;
pub(crate) mod timer;
include!("pc.rs");
