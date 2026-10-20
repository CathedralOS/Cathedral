//! Queue custody for sealed-memory grants. Memory owns names, quotas and backing;
//! IPC carries an opaque escrow slot and reports discarded slots for retirement.
use super::*;
impl Ipc {
    pub fn memory_receiver(&self, caller: usize, ticket: u64, length: usize) -> Result<usize, u64> {
        let index = self.check(caller, ticket, Right::Send)?;
        if length > abi::MAX_MESSAGE {
            return Err(abi::INVALID_ARGUMENT);
        }
        let endpoint = self.endpoints[index].as_ref().unwrap();
        if !endpoint.receiver_alive {
            return Err(abi::PEER_CLOSED);
        }
        if endpoint.message.is_some() {
            return Err(abi::WOULD_BLOCK);
        }
        Ok(endpoint.spec.receiver)
    }
    /// Caller reserved a recipient grant after memory_receiver; serialized commit.
    pub fn send_memory(&mut self, caller: usize, ticket: u64, bytes: &[u8], escrow: usize) {
        assert!(escrow < cathedral_kernel_contracts::memory::MAX_READ_HANDLES);
        self.memory_receiver(caller, ticket, bytes.len())
            .expect("validated memory send");
        let index = self.check(caller, ticket, Right::Send).unwrap();
        let mut message = Queued {
            bytes: [0; abi::MAX_MESSAGE],
            len: bytes.len(),
            attachment: None,
            memory: Some(escrow),
        };
        message.bytes[..bytes.len()].copy_from_slice(bytes);
        self.endpoints[index].as_mut().unwrap().message = Some(message);
    }
    pub fn memory_retirement_pending(&self) -> bool {
        self.retired_memory.iter().any(|&word| word != 0)
    }
    pub fn take_retired_memory(
        &mut self,
    ) -> [u64; cathedral_kernel_contracts::memory::MAX_READ_HANDLES.div_ceil(64)] {
        core::mem::take(&mut self.retired_memory)
    }
}
