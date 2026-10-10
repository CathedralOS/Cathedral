#![no_std]
#![forbid(unsafe_code)]
//! Userspace LBA28 single-sector PIO on the boot-granted secondary ATA master.
use cathedral_contracts::{block::Device, user as abi};
use cathedral_user_runtime::{Error, disk, time, yield_now};
pub struct Ata {
    sectors: u64,
}
fn code(error: Error) -> u64 {
    error.0 as u64
}
fn read(reg: u64) -> Result<u8, u64> {
    disk::read(reg).map_err(code)
}
fn write(reg: u64, byte: u8) -> Result<(), u64> {
    disk::write(reg, byte).map_err(code)
}
fn now() -> Result<u64, u64> {
    time::now().map_err(code)
}
fn wait(data: bool) -> Result<(), u64> {
    let deadline = time::after(50).map_err(code)?;
    loop {
        let status = read(8)?;
        if status == 0 || status == 255 {
            return Err(abi::IO_ERROR);
        }
        if status & 0x80 == 0 {
            if status & 0x21 != 0 {
                return Err(abi::IO_ERROR);
            }
            if (status & 8 != 0) == data {
                return Ok(());
            }
        }
        if time::reached(now()?, deadline) {
            return Err(abi::TIMED_OUT);
        }
        yield_now();
    }
}
fn delay() -> Result<(), u64> {
    for _ in 0..4 {
        read(8)?;
    }
    Ok(())
}
impl Ata {
    pub fn open() -> Result<Self, u64> {
        write(8, 6)?; // Recover an interrupted PIO command; never issue DMA.
        let deadline = time::after(1).map_err(code)?;
        while !time::reached(now()?, deadline) {
            yield_now();
        }
        write(8, 2)?;
        write(6, 0xe0)?;
        delay()?;
        wait(false)?;
        write(7, 0xec)?;
        delay()?;
        wait(true)?;
        let mut identify = [0; 512];
        disk::transfer(&mut identify, false).map_err(code)?;
        wait(false)?;
        let word = |i: usize| u16::from_le_bytes([identify[i * 2], identify[i * 2 + 1]]);
        if word(49) & (1 << 9) == 0 || word(83) & (1 << 12) == 0 {
            return Err(abi::IO_ERROR);
        }
        let mut serial = [0; 20];
        for (output, input) in serial
            .chunks_exact_mut(2)
            .zip(identify[20..40].chunks_exact(2))
        {
            output[0] = input[1];
            output[1] = input[0];
        }
        if &serial != b"CATHEDRAL-LAB-DATA  " {
            return Err(abi::DENIED);
        }
        let sectors = u64::from(word(60)) | (u64::from(word(61)) << 16);
        if !(4..1 << 28).contains(&sectors) {
            return Err(abi::IO_ERROR);
        }
        Ok(Self { sectors })
    }
    fn command(&self, sector: u64, op: u8) -> Result<(), u64> {
        if sector >= self.sectors {
            return Err(abi::INVALID_ARGUMENT);
        }
        wait(false)?;
        write(6, 0xe0 | ((sector >> 24) as u8 & 15))?;
        delay()?;
        write(2, 1)?;
        for (reg, shift) in [(3, 0), (4, 8), (5, 16)] {
            write(reg, (sector >> shift) as u8)?;
        }
        write(7, op)?;
        delay()?;
        wait(true)
    }
}
impl Device for Ata {
    fn sectors(&self) -> u64 {
        self.sectors
    }
    fn read(&mut self, sector: u64, bytes: &mut [u8; 512]) -> Result<(), u64> {
        self.command(sector, 0x20)?;
        disk::transfer(bytes, false).map_err(code)?;
        wait(false)
    }
    fn write(&mut self, sector: u64, bytes: &[u8; 512]) -> Result<(), u64> {
        self.command(sector, 0x30)?;
        let mut buffer = *bytes;
        disk::transfer(&mut buffer, true).map_err(code)?;
        wait(false)
    }
    fn flush(&mut self) -> Result<(), u64> {
        wait(false)?;
        write(7, 0xe7)?;
        delay()?;
        wait(false)
    }
}
