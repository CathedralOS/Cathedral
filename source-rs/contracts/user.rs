//! Experimental Rust-lab ABI, not Cathedral's accepted component contract.
//! x86-64: INT 0x80; RAX is call/result, RDI/RSI are arguments. Entry receives
//! two ordinary arguments in RDI/RSI and a 16-byte-aligned stack without a caller.
//! Write accepts at most 256 bytes within one mapped user page.

pub const WRITE: u64 = 0;
pub const YIELD: u64 = 1;
pub const EXIT: u64 = 2;
pub const MAX_WRITE: usize = 256;
pub const IO_ERROR: u64 = (-5i64) as u64;
pub const BAD_ADDRESS: u64 = (-14i64) as u64;
pub const INVALID_ARGUMENT: u64 = (-22i64) as u64;
pub const UNKNOWN: u64 = (-38i64) as u64;
