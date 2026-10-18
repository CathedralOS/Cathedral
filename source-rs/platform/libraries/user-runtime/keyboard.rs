//! Exclusive bootstrap PS/2 byte channel, available only to its granted child.
//! This temporary interface exposes fixed controller ports, never arbitrary I/O.
use crate::{Error, abi, arch};

pub const LOST: u64 = abi::KEYBOARD_LOST;
/// Auxiliary-controller byte tag; low eight bits carry the mouse byte.
pub const AUX: u64 = abi::KEYBOARD_AUX;
pub fn read_until(deadline: u64) -> Result<u64, Error> {
    result(arch::call(abi::KEYBOARD_READ_UNTIL, deadline, 0))
}
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

/// Restricted virtual-mouse command under the same exclusive input grant.
pub fn virtual_pointer(command: u64, data: u64) -> Result<[u32; 4], Error> {
    let mut reply = [0u32; 4];
    result(arch::call3(
        abi::INPUT_VIRTUAL_POINTER,
        command,
        data,
        reply.as_mut_ptr() as u64,
    ))?;
    Ok(reply)
}
