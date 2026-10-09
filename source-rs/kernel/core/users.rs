//! Experimental hardware-contained tasks. Not Cathedral's frozen component ABI.
//! CPU mappings/entry live in arch; syscall policy and task lifetime live here.

pub mod elf;
mod runtime;
pub mod syscall;
pub use runtime::{Exit, Report, run};

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
    OutOfHeap,
    Executable(elf::Error),
    Memory(cathedral_arch::MemoryError),
}
