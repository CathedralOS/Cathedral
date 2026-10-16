//! Synchronous bounded block backend used by the storage experiment.
//! Successful flush persists all preceding writes; errors never promise durability.
//! Read/write completion alone promises no persistence or sector atomicity. A failed
//! write or flush may have persisted some or all bytes; callers must treat its
//! outcome as uncertain, recover the device, and reopen durable state. Sector count
//! remains fixed during exclusive ownership. Transport reset is not a flush.
pub const SECTOR: usize = 512;
pub trait Device {
    fn sectors(&self) -> u64;
    fn read(&mut self, sector: u64, bytes: &mut [u8; SECTOR]) -> Result<(), u64>;
    fn write(&mut self, sector: u64, bytes: &[u8; SECTOR]) -> Result<(), u64>;
    fn flush(&mut self) -> Result<(), u64>;
}
