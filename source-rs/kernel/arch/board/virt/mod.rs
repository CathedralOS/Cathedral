mod console;
pub(crate) mod gic;
pub use console::Console;
include!("virt.rs");
