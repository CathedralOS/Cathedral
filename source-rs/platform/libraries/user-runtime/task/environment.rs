//! Launch connections installed before execution. Roles/order are an app protocol,
//! not kernel service names. Kernel route: session/syscalls/tasks/launch.rs.
use super::{Child, Error, Handle, Launch, abi, arch, result};
#[derive(Debug)]
pub struct Environment {
    handles: [u64; abi::MAX_INITIAL_ENDPOINTS],
    count: usize,
}
impl Environment {
    /// Snapshot initial names only; reading again cannot revive closed/moved grants.
    pub fn load() -> Result<Self, Error> {
        let mut record = Record([0; 64]);
        result(arch::call3(
            abi::TASK_ENVIRONMENT,
            record.0.as_mut_ptr() as u64,
            0,
            0,
        ))?;
        let count = u64::from_le_bytes(record.0[..8].try_into().unwrap());
        if count > abi::MAX_INITIAL_ENDPOINTS as u64 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        let mut handles = [0; abi::MAX_INITIAL_ENDPOINTS];
        for (h, bytes) in handles
            .iter_mut()
            .zip(record.0[8..abi::ENVIRONMENT_BYTES].chunks_exact(8))
        {
            *h = u64::from_le_bytes(bytes.try_into().unwrap());
        }
        Ok(Self {
            handles,
            count: count as usize,
        })
    }
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn endpoint(&self, index: usize) -> Option<Handle> {
        self.handles[..self.count]
            .get(index)
            .copied()
            .map(Handle::from_raw)
    }
}
impl Launch {
    /// Move a whole ordered bundle on successful admission. Every failure retains
    /// all endpoints; success invalidates every parent-side copy of those names.
    /// Dynamic IPC endpoints only. Profile device/clock grants remain separate.
    pub fn spawn_with(self, argument: u64, endpoints: &[Handle]) -> Result<Child, Error> {
        if endpoints.len() > abi::MAX_INITIAL_ENDPOINTS {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut record = Record([0; 64]);
        record.0[..8].copy_from_slice(&argument.to_le_bytes());
        record.0[8..16].copy_from_slice(&(endpoints.len() as u64).to_le_bytes());
        for (endpoint, bytes) in endpoints
            .iter()
            .zip(record.0[16..abi::SPAWN_BYTES].chunks_exact_mut(8))
        {
            bytes.copy_from_slice(&endpoint.raw().to_le_bytes());
        }
        result(arch::call3(
            abi::TASK_SPAWN_WITH,
            self.raw(),
            record.0.as_ptr() as u64,
            0,
        ))
        .map(Child)
    }
}
#[repr(align(64))]
struct Record([u8; 64]);
