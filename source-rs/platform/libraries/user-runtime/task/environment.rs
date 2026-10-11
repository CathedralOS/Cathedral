//! Launch connections installed before execution. Roles/order are an app protocol,
//! not kernel service names. Kernel route: session/syscalls/tasks/launch.rs.
use super::{Error, Handle, abi, arch, result};
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
#[repr(align(64))]
struct Record([u8; 64]);
