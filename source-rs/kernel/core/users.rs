//! Experimental hardware-contained tasks. Not Cathedral's frozen component ABI.
//! CPU mappings/entry live in arch; syscall policy and task lifetime live here.

mod runtime;
pub mod syscall;
pub use runtime::{Exit, Report, run};

pub struct Program<'a> {
    pub code: &'a [u8],
    /// Initial RDI/RSI values; ordinary data, not capabilities.
    pub arguments: [u64; 2],
}

#[derive(Debug)]
pub enum Error {
    InvalidCount,
    OutOfHeap,
    Memory(cathedral_arch::MemoryError),
}
