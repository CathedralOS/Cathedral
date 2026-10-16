//! Arbitrary byte IO over exclusively borrowed sectors. Validate the whole range
//! before IO; partial writes preserve neighbouring bytes with read/modify/write.
use crate::Error;
use cathedral_contracts::block::{Device, SECTOR};

pub struct BlockRegion<'a, D> {
    device: &'a mut D,
    start: u64,
    bytes: u64,
    failed: bool,
}
impl<'a, D: Device> BlockRegion<'a, D> {
    pub fn new(device: &'a mut D, start: u64, bytes: u64) -> Result<Self, Error> {
        let extent = device.sectors().checked_mul(SECTOR as u64);
        if bytes == 0
            || !start.is_multiple_of(SECTOR as u64)
            || !bytes.is_multiple_of(SECTOR as u64)
            || extent.is_none()
            || start
                .checked_add(bytes)
                .is_none_or(|end| end > extent.unwrap())
        {
            return Err(Error::Invalid("unaligned or out-of-device region"));
        }
        Ok(Self {
            device,
            start,
            bytes,
            failed: false,
        })
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    fn checked(&self, offset: u64, bytes: usize) -> Result<(), Error> {
        if self.failed {
            return Err(Error::Failed);
        }
        if offset
            .checked_add(bytes as u64)
            .is_none_or(|end| end > self.bytes)
        {
            return Err(Error::Invalid("IO exceeds assigned region"));
        }
        Ok(())
    }
    pub fn read(&mut self, mut offset: u64, mut out: &mut [u8]) -> Result<(), Error> {
        self.checked(offset, out.len())?;
        let mut sector = [0; SECTOR];
        while !out.is_empty() {
            let physical = self.start + offset;
            let within = physical as usize % SECTOR;
            let count = out.len().min(SECTOR - within);
            self.device
                .read(physical / SECTOR as u64, &mut sector)
                .map_err(Error::Device)?;
            out[..count].copy_from_slice(&sector[within..within + count]);
            offset += count as u64;
            out = &mut out[count..];
        }
        Ok(())
    }
    pub fn write(&mut self, mut offset: u64, mut bytes: &[u8]) -> Result<(), Error> {
        self.checked(offset, bytes.len())?;
        let result = (|| {
            let mut sector = [0; SECTOR];
            while !bytes.is_empty() {
                let physical = self.start + offset;
                let within = physical as usize % SECTOR;
                let count = bytes.len().min(SECTOR - within);
                if within != 0 || count != SECTOR {
                    self.device.read(physical / SECTOR as u64, &mut sector)?;
                }
                sector[within..within + count].copy_from_slice(&bytes[..count]);
                self.device.write(physical / SECTOR as u64, &sector)?;
                offset += count as u64;
                bytes = &bytes[count..];
            }
            Ok(())
        })();
        self.finish(result)
    }
    /// A write completion is not durable. Only this device barrier promises
    /// persistence of preceding successful writes, including partial-sector IO.
    pub fn flush(&mut self) -> Result<(), Error> {
        self.checked(0, 0)?;
        let result = self.device.flush();
        self.finish(result)
    }
    fn finish(&mut self, result: Result<(), u64>) -> Result<(), Error> {
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(Error::Device)
    }
}
