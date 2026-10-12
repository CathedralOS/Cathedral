//! CPU-to-core event vocabulary. Raw fault vector/syndrome values retain their architecture meaning.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SwitchCause {
    Timer,
    Device,
    Yield,
    Syscall,
    Fault(UserFault),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserFault {
    pub vector: u64,
    pub error: u64,
    pub address: u64,
    pub instruction: u64,
}
#[derive(Debug)]
pub struct Fault {
    pub vector: u64,
    pub error: u64,
    pub address: u64,
    pub instruction: u64,
    pub stack: u64,
}
