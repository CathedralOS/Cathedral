//! Boot-granted secondary ATA register and one-sector data transport.
use crate::{Error, abi, arch};
pub fn read(register: u64) -> Result<u8, Error> {
    result(arch::call(abi::DISK_REGISTER_READ, register, 0)).map(|v| v as u8)
}
pub fn write(register: u64, value: u8) -> Result<(), Error> {
    result(arch::call(
        abi::DISK_REGISTER_WRITE,
        register,
        u64::from(value),
    ))
    .map(|_| ())
}
pub fn transfer(bytes: &mut [u8; 512], write: bool) -> Result<(), Error> {
    #[repr(align(512))]
    struct Buffer([u8; 512]);
    let mut buffer = Buffer(*bytes);
    result(arch::call3(
        abi::DISK_TRANSFER,
        buffer.0.as_mut_ptr() as u64,
        u64::from(write),
        512,
    ))?;
    if !write {
        *bytes = buffer.0;
    }
    Ok(())
}
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
