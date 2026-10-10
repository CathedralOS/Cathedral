//! Synchronous bounded block backend used by the storage experiment.
//! Successful flush persists all preceding writes; errors never promise durability.
pub const SECTOR: usize = 512;
pub trait Device {
    fn sectors(&self) -> u64;
    fn read(&mut self, sector: u64, bytes: &mut [u8; SECTOR]) -> Result<(), u64>;
    fn write(&mut self, sector: u64, bytes: &[u8; SECTOR]) -> Result<(), u64>;
    fn flush(&mut self) -> Result<(), u64>;
}
