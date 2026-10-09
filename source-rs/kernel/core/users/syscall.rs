//! Rust-lab ABI: INT 0x80, RAX=number/result, RDI/RSI/RDX=arguments.
//! Write copies at most 256 bytes within one mapped user page. Errors are
//! negative values in RAX; no ambient user pointer ever reaches a driver.

pub use cathedral_contracts::user::*;
