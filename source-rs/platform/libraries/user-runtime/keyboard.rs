//! Exclusive bootstrap PS/2 byte channel, available only to its granted child.
//! This temporary interface exposes fixed controller ports, never arbitrary I/O.
use crate::{Error, abi, arch};

pub const LOST: u64 = 256;
pub fn read(wait: bool) -> Result<u64, Error> {
    result(arch::call(abi::KEYBOARD_READ, u64::from(wait), 0))
}
pub fn write(command: bool, byte: u8) -> Result<(), Error> {
    result(arch::call(
        abi::KEYBOARD_WRITE,
        u64::from(command),
        u64::from(byte),
    ))
    .map(|_| ())
}
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
