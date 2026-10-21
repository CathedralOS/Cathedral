// Physical-frame custody and mapping intent shared by CPU backends and kernel owners.
// UserMappings is selected statically by arch/lib.rs; it owns translations, never frame lifetime.
pub const PAGE: u64 = 4096;
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryError {
    UnsupportedCpu,
    OutOfFrames,
    InvalidFrame,
    TableBudget,
    OccupiedVirtualRange,
    MapFailed,
    InvalidImage,
}
/// # Safety
/// Allocations are unique, aligned, unused RAM valid for the selected board and
/// writable through the kernel's physical alias. Release requires every live use retired.
pub unsafe trait FrameSource {
    fn allocate(&mut self) -> Option<u64>;
    /// # Safety
    /// Address came from this source; all users and non-permanent aliases have retired.
    unsafe fn release(&mut self, address: u64);
}
#[derive(Clone, Copy)]
pub struct ImageRange {
    pub base: u64,
    pub bytes: u64,
}
#[derive(Clone, Copy)]
pub struct Permissions {
    pub writable: bool,
    pub executable: bool,
}
/// Geometry only; constructing this record does not grant ownership.
#[derive(Clone, Copy, Debug)]
pub struct StackRange {
    pub bottom: u64,
    pub top: u64,
    pub guard: u64,
}
