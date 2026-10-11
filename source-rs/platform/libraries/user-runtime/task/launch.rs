//! Admission requests: endpoint moves and bounded recursive launch loans.
//! Kernel route: session/syscalls/tasks/launch.rs, then lifecycle::spawn.
use super::{Child, Error, Handle, Launch, abi, arch, result};
impl Launch {
    /// Move a whole ordered bundle on successful admission. Every failure retains
    /// all endpoints; success invalidates every parent-side copy of those names.
    /// Dynamic IPC endpoints only. Profile device/clock grants remain separate.
    pub fn spawn_with(self, argument: u64, endpoints: &[Handle]) -> Result<Child, Error> {
        self.bundle(abi::TASK_SPAWN_WITH, argument, endpoints, &[])
    }
    /// Lend selected idle launch budgets to this child, including their peer ports.
    /// Budgets return with fresh names after the child's subtree is reclaimed;
    /// use Launch::at to reacquire them. A budget's executable/ceilings stay fixed.
    /// The child can enumerate only its loans and can lend them further.
    pub fn spawn_tree(
        self,
        argument: u64,
        endpoints: &[Handle],
        launches: &[Launch],
    ) -> Result<Child, Error> {
        self.bundle(abi::TASK_SPAWN_TREE, argument, endpoints, launches)
    }
    fn bundle(
        self,
        number: u64,
        argument: u64,
        endpoints: &[Handle],
        launches: &[Launch],
    ) -> Result<Child, Error> {
        if endpoints.len() > abi::MAX_INITIAL_ENDPOINTS || launches.len() > abi::MAX_LAUNCHES {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut record = Record([0; 128]);
        record.0[..8].copy_from_slice(&argument.to_le_bytes());
        record.0[8..16].copy_from_slice(&(endpoints.len() as u64).to_le_bytes());
        for (endpoint, bytes) in endpoints
            .iter()
            .zip(record.0[16..abi::SPAWN_BYTES].chunks_exact_mut(8))
        {
            bytes.copy_from_slice(&endpoint.raw().to_le_bytes());
        }
        record.0[abi::SPAWN_BYTES..abi::SPAWN_BYTES + 8]
            .copy_from_slice(&(launches.len() as u64).to_le_bytes());
        for (launch, bytes) in launches
            .iter()
            .zip(record.0[abi::SPAWN_BYTES + 8..abi::SPAWN_TREE_BYTES].chunks_exact_mut(8))
        {
            bytes.copy_from_slice(&launch.raw().to_le_bytes());
        }
        result(arch::call3(number, self.raw(), record.0.as_ptr() as u64, 0)).map(Child)
    }
}
#[repr(align(128))]
struct Record([u8; 128]);
