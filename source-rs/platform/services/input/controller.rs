//! Configure the QEMU bootstrap controller; decoding lives independently above it.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, keyboard, yield_now};

fn write(command: bool, byte: u8) -> Result<(), Error> {
    for _ in 0..1024 {
        match keyboard::write(command, byte) {
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => yield_now(),
            result => return result,
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
fn device(byte: u8) -> Result<(), Error> {
    for _ in 0..3 {
        write(false, byte)?;
        match keyboard::read(true)? {
            0xfa => return Ok(()),
            0xfe => continue,
            _ => break,
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
pub fn initialize() -> Result<(), Error> {
    write(true, 0xad)?;
    write(true, 0xa7)?;
    for _ in 0..65 {
        match keyboard::read(false) {
            Ok(_) => (),
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => break,
            Err(error) => return Err(error),
        }
    }
    write(true, 0x20)?;
    let mode = keyboard::read(true)?;
    if mode > 255 {
        return Err(Error(abi::IO_ERROR as i64));
    }
    write(true, 0x60)?;
    write(false, (mode as u8 | 0x61) & !0x12)?;
    write(true, 0xae)?;
    device(0xf5)?; // stop scanning while selecting set 2
    device(0xf0)?;
    device(2)?;
    device(0xf4)?;
    Ok(())
}
