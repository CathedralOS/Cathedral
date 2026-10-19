#![no_std]
#![forbid(unsafe_code)]
//! Synchronous bounded block backend used by the storage experiment.
//! Successful flush persists all preceding writes; errors never promise durability.
//! Read/write completion alone promises no persistence or sector atomicity. A failed
//! write or flush may have persisted some or all bytes; callers must treat its
//! outcome as uncertain, recover the device, and reopen durable state. Sector count
//! remains fixed during exclusive ownership. Transport reset is not a flush.
pub const SECTOR: usize = 512;
/// Invalid contiguous range, rejected before issuing any device operation.
pub const OUT_OF_RANGE: u64 = u64::MAX;
pub trait Device {
    fn sectors(&self) -> u64;
    fn read(&mut self, sector: u64, bytes: &mut [u8; SECTOR]) -> Result<(), u64>;
    fn write(&mut self, sector: u64, bytes: &[u8; SECTOR]) -> Result<(), u64>;
    /// Contiguous sectors; adapters may batch commands while preserving failure semantics.
    fn read_many(&mut self, sector: u64, out: &mut [[u8; SECTOR]]) -> Result<(), u64> {
        if sector
            .checked_add(out.len() as u64)
            .is_none_or(|end| end > self.sectors())
        {
            return Err(OUT_OF_RANGE);
        }
        for (index, bytes) in out.iter_mut().enumerate() {
            self.read(sector + index as u64, bytes)?;
        }
        Ok(())
    }
    fn write_many(&mut self, sector: u64, bytes: &[[u8; SECTOR]]) -> Result<(), u64> {
        if sector
            .checked_add(bytes.len() as u64)
            .is_none_or(|end| end > self.sectors())
        {
            return Err(OUT_OF_RANGE);
        }
        for (index, bytes) in bytes.iter().enumerate() {
            self.write(sector + index as u64, bytes)?;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), u64>;
}
