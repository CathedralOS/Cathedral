//! Experimental hardware-contained tasks. Not Cathedral's frozen component ABI.
//! CPU mappings/entry live in arch; syscall policy and task lifetime live here.
//! Start at [`run_configured`]: session.rs exposes validation, preparation,
//! admission and execution. Configuration and returned outcomes have separate owners.

mod configuration;
pub mod elf;
mod outcomes;
mod session;
pub mod syscall;
pub use configuration::{Config, Supervision};
pub use outcomes::{Exit, MemoryReport, Report};
#[cfg(feature = "memory-lab")]
pub use session::exercise_memory_rollback;
pub use session::{run, run_configured};

#[derive(Clone, Copy)]
pub enum Executable<'a> {
    Probe(&'a [u8]),
    Elf(&'a [u8]),
}
pub struct Program<'a> {
    pub executable: Executable<'a>,
    /// Initial RDI/RSI values; ordinary data, not capabilities.
    pub arguments: [u64; 2],
}

#[derive(Debug)]
pub enum Error {
    InvalidCount,
    InvalidEndpoints,
    OutOfHeap,
    Executable(elf::Error),
    Memory(cathedral_arch::MemoryError),
}
