//! Configure the QEMU bootstrap controller; decoding lives independently above it.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{Error, keyboard, time, yield_now};

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
        let deadline = time::after(50)?;
        loop {
            match keyboard::read_until(deadline)? {
                0xfa => return Ok(()),
                0xfe => break,
                keyboard::LOST => return Err(Error(abi::IO_ERROR as i64)),
                // A restarted source can encounter held-key repeats before ACK.
                _ => (),
            }
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
pub fn initialize() -> Result<(), Error> {
    configure_keyboard()?;
    device(0xf4)
}
/// Return the configured controller mode with keyboard scanning still stopped.
fn configure_keyboard() -> Result<u8, Error> {
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
    let mode = keyboard::read_until(time::after(50)?)?;
    if mode > 255 {
        return Err(Error(abi::IO_ERROR as i64));
    }
    write(true, 0x60)?;
    let mode = (mode as u8 | 0x61) & !0x12;
    write(false, mode)?;
    write(true, 0xae)?;
    device(0xf5)?; // stop scanning while selecting set 2
    device(0xf0)?;
    device(2)?;
    Ok(mode)
}

/// Enable the standard three-byte mouse with auxiliary IRQ disabled. The bounded
/// kernel timer poll services both channels; mouse decoding stays in userspace.
pub fn initialize_mouse() -> Result<(), Error> {
    let mode = configure_keyboard()?;
    write(true, 0xa8)?;
    // Reuse the mode read while both interfaces were disabled. Reading it here
    // would race queued auxiliary reports or keyboard repeats after a restart.
    write(true, 0x60)?;
    write(false, (mode | 0x41) & !0x32)?;
    for command in [0xf5, 0xf6, 0xe6, 0xea, 0xf4] {
        let mut accepted = false;
        for _ in 0..3 {
            write(true, 0xd4)?;
            write(false, command)?;
            let deadline = time::after(50)?;
            loop {
                let byte = keyboard::read_until(deadline)?;
                if byte == keyboard::AUX + 0xfa {
                    accepted = true;
                    break;
                }
                if byte == keyboard::AUX + 0xfe {
                    break;
                }
                if byte == keyboard::LOST {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                // Ignore pending keyboard reports during configuration.
            }
            if accepted {
                break;
            }
        }
        if !accepted {
            return Err(Error(abi::IO_ERROR as i64));
        }
    }
    device(0xf4)
}
