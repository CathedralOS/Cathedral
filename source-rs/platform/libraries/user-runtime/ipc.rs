//! Boot-issued, fixed-right lab endpoints. Copies are bounded; receive may park.
//! No discovery, delegation, ownership transfer or stable production ABI.
use crate::{Error, abi, arch};

#[derive(Clone, Copy, Debug)]
pub struct Handle(u64);
impl Handle {
    pub fn bootstrap(index: u64) -> Result<Self, Error> {
        result(arch::call(abi::IPC_HANDLE, index, 0)).map(Self)
    }
    /// Raw bits carry no authority outside the task to which the grant was issued.
    pub fn from_raw(bits: u64) -> Self {
        Self(bits)
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    pub fn send(self, bytes: &[u8]) -> Result<(), Error> {
        // Stage in a page-contained buffer; the caller's slice may cross pages.
        if bytes.len() > abi::MAX_MESSAGE {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut buffer = Buffer([0; abi::MAX_MESSAGE]);
        buffer.0[..bytes.len()].copy_from_slice(bytes);
        result(arch::call3(
            abi::IPC_SEND,
            self.0,
            buffer.0.as_ptr() as u64,
            bytes.len() as u64,
        ))
        .map(|_| ())
    }
    pub fn receive(self, bytes: &mut [u8]) -> Result<usize, Error> {
        let capacity = bytes.len().min(abi::MAX_MESSAGE);
        let mut buffer = Buffer([0; abi::MAX_MESSAGE]);
        let length = result(arch::call3(
            abi::IPC_RECEIVE,
            self.0,
            buffer.0.as_mut_ptr() as u64,
            capacity as u64,
        ))? as usize;
        if length > capacity {
            return Err(Error(abi::IO_ERROR as i64));
        }
        bytes[..length].copy_from_slice(&buffer.0[..length]);
        Ok(length)
    }
    pub fn revoke(self) -> Result<(), Error> {
        result(arch::call(abi::IPC_REVOKE, self.0, 0)).map(|_| ())
    }
    /// Timeout cancels only this receive, not the channel or remote request.
    /// Ready data and terminal errors take precedence over expiry. Applications
    /// must drain/correlate late replies or replace the connection before reuse.
    pub fn receive_until(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
        deadline: u64,
    ) -> Result<usize, Error> {
        let mut buffer = Buffer([0; abi::MAX_MESSAGE]);
        let length = result(arch::call3(
            abi::IPC_RECEIVE_UNTIL,
            self.0,
            buffer.0.as_mut_ptr() as u64,
            deadline,
        ))? as usize;
        if length > abi::MAX_MESSAGE {
            return Err(Error(abi::IO_ERROR as i64));
        }
        bytes[..length].copy_from_slice(&buffer.0[..length]);
        Ok(length)
    }
}
// Alignment equals size, and 64 divides a page: the buffer cannot straddle one.
#[repr(align(64))]
struct Buffer([u8; abi::MAX_MESSAGE]);
/// Wait without consuming data; result 0/1 selects the ready receiver. A peer
/// close is readiness so receive can report its terminal status. Requires clock.
pub fn wait_two(first: Handle, second: Handle, deadline: u64) -> Result<usize, Error> {
    result(arch::call3(
        abi::IPC_WAIT_TWO,
        first.raw(),
        second.raw(),
        deadline,
    ))
    .map(|index| index as usize)
}
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
