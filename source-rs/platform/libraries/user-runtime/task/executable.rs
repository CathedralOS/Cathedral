//! Code authority and funded process reservations. No ambient binary lookup.
use super::*;
use crate::resources::Limits;
#[derive(Clone, Copy, Debug)]
pub struct Executable(u64);
impl Executable {
    /// Dynamic admission explicitly supplies its executable for bounded descendants.
    pub fn at(index: u64) -> Result<Self, Error> {
        result(arch::call(abi::TASK_IMAGE_AT, index, 0)).map(Self)
    }
    pub fn from_raw(name: u64) -> Self {
        Self(name)
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    pub fn reserve(
        self,
        limits: Limits,
        child_slots: u64,
        admission_frames: u64,
    ) -> Result<Launch, Error> {
        #[repr(align(64))]
        struct Request([u64; 5]);
        let request = Request([
            limits.private_pages,
            limits.reader_pages,
            limits.channels,
            child_slots,
            admission_frames,
        ]);
        result(arch::call3(
            abi::TASK_RESERVE,
            self.0,
            request.0.as_ptr() as u64,
            0,
        ))
        .map(Launch)
    }
}
pub fn available_slots() -> Result<u64, Error> {
    result(arch::call(abi::TASK_POOL_AVAILABLE, 0, 0))
}
impl Launch {
    pub fn executable(self) -> Result<Executable, Error> {
        result(arch::call(abi::TASK_EXECUTABLE, self.0, 0)).map(Executable)
    }
    pub fn assign_slots(self, count: u64) -> Result<(), Error> {
        result(arch::call(abi::TASK_POOL_ASSIGN, self.0, count)).map(|_| ())
    }
    pub fn port(self) -> Result<Port, Error> {
        result(arch::call(abi::TASK_LAUNCH_PORT, self.0, 0)).map(Port)
    }
    /// Only idle dynamic accounts; fails while retained backing pins their funding.
    pub fn release(self) -> Result<(), Error> {
        result(arch::call(abi::TASK_RELEASE, self.0, 0)).map(|_| ())
    }
}
