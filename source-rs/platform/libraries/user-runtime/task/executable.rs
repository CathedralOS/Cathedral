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
    /// Code from a sealed object this task can read, entered with `role` as its
    /// first argument. The grant is consumed by one reservation and admitted
    /// once; the object may be released as soon as this returns.
    pub fn from_object(object: &crate::memory::Object, role: u64) -> Result<Self, Error> {
        result(arch::call(
            abi::TASK_IMAGE_FROM_OBJECT,
            object.handle(),
            role,
        ))
        .map(Self)
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
        let physical_frames = child_slots
            .checked_add(1)
            .and_then(|count| count.checked_mul(abi::MAX_ADMISSION_FRAMES as u64))
            .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        self.reserve_funded(limits, child_slots, admission_frames, physical_frames)
    }
    /// Reserve real frame capacity separately from the per-load failure ceiling.
    pub fn reserve_funded(
        self,
        limits: Limits,
        child_slots: u64,
        admission_frames: u64,
        physical_frames: u64,
    ) -> Result<Launch, Error> {
        #[repr(align(64))]
        struct Request([u64; 6]);
        let request = Request([
            limits.private_pages,
            limits.reader_pages,
            limits.channels,
            child_slots,
            admission_frames,
            physical_frames,
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
    pub fn allow_executable(self, image: Executable) -> Result<(), Error> {
        result(arch::call(abi::TASK_IMAGE_ALLOW, self.0, image.raw())).map(|_| ())
    }
    pub fn assign_memory(self, frames: u64) -> Result<(), Error> {
        result(arch::call(abi::PHYSICAL_ASSIGN, self.0, frames)).map(|_| ())
    }
    pub fn assign_cpu(self, limit_us: u64) -> Result<(), Error> {
        result(arch::call(abi::TASK_CPU_ASSIGN, self.0, limit_us)).map(|_| ())
    }

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
