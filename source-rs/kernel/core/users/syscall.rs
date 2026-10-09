//! Rust-lab ABI: INT 0x80, RAX=number/result, RDI/RSI=arguments.
//! Write copies at most 256 bytes within one mapped user page. Errors are
//! negative values in RAX; no ambient user pointer ever reaches a driver.

pub const WRITE: u64 = 0;
pub const YIELD: u64 = 1;
pub const EXIT: u64 = 2;
pub const MAX_WRITE: usize = 256;
pub const IO_ERROR: u64 = (-5i64) as u64;
pub const BAD_ADDRESS: u64 = (-14i64) as u64;
pub const INVALID_ARGUMENT: u64 = (-22i64) as u64;
pub const UNKNOWN: u64 = (-38i64) as u64;
