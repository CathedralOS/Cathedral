//! QEMU virtual mouse negotiation and bounded packet reads; coordinates stay local.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{Error, keyboard, write};
pub struct Pointer {
    enabled: bool,
}
impl Pointer {
    pub fn open() -> Self {
        let enabled = Self::enable().is_ok();
        if enabled {
            let _ = write(b"Cathedral input: absolute pointer ready\n");
        }
        Self { enabled }
    }
    fn enable() -> Result<(), Error> {
        let version = keyboard::virtual_pointer(10, 0)?;
        if version[1] != 0x564d5868 || version[0] == u32::MAX {
            return Err(Error(abi::DENIED as i64));
        }
        keyboard::virtual_pointer(41, 0xf5)?;
        // Drain the bounded old queue before asking for the protocol identity.
        for _ in 0..1024 {
            let status = keyboard::virtual_pointer(40, 0)?[0];
            if status & 0xffff == 0 {
                break;
            }
            keyboard::virtual_pointer(39, 1)?;
        }
        keyboard::virtual_pointer(41, 0x45414552)?;
        if keyboard::virtual_pointer(40, 0)?[0] != 1
            || keyboard::virtual_pointer(39, 1)?[0] != 0x3442554a
        {
            let _ = keyboard::virtual_pointer(41, 0xf5);
            return Err(Error(abi::IO_ERROR as i64));
        }
        keyboard::virtual_pointer(41, 0x53424152)?;
        Ok(())
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn next(&mut self, width: u64, height: u64) -> Result<Option<(u64, u64, u64)>, Error> {
        if !self.enabled {
            return Ok(None);
        }
        let status = keyboard::virtual_pointer(40, 0)?[0];
        if status >> 16 != 0 || status & 3 != 0 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        if status & 0xffff == 0 {
            return Ok(None);
        }
        let [flags, x, y, _] = keyboard::virtual_pointer(39, 4)?;
        if flags & 0x10000 != 0 || x > 65535 || y > 65535 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        let buttons = u64::from(flags & 0x20 != 0)
            | (u64::from(flags & 0x10 != 0) << 1)
            | (u64::from(flags & 8 != 0) << 2);
        Ok(Some((
            (u64::from(x) * (width - 1) + 32767) / 65535,
            (u64::from(y) * (height - 1) + 32767) / 65535,
            buttons,
        )))
    }
    pub fn disable(&mut self) {
        let _ = keyboard::virtual_pointer(41, 0xf5);
        self.enabled = false;
    }
}
