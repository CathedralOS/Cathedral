//! Transitional typed handoff: source retains its right; explicit receive accepts escrow.
use super::*;
use crate::ipc::Handle;
impl Notification {
    pub fn delegate(self, endpoint: Handle, bytes: &[u8]) -> Result<(), Error> {
        if bytes.len() > abi::MAX_MESSAGE {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut packet = Record([0; abi::IPC_PACKET_BYTES]);
        packet.0[..8].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
        packet.0[8..16].copy_from_slice(&self.raw().to_le_bytes());
        packet.0[16..16 + bytes.len()].copy_from_slice(bytes);
        result(arch::call3(
            abi::NOTIFY_SEND,
            endpoint.raw(),
            packet.0.as_ptr() as u64,
            0,
        ))
        .map(|_| ())
    }
    pub fn accept(
        endpoint: Handle,
        bytes: &mut [u8; abi::MAX_MESSAGE],
    ) -> Result<(usize, Option<Self>), Error> {
        Self::receive(endpoint, bytes, abi::NOTIFY_RECEIVE)
    }
    pub fn try_accept(
        endpoint: Handle,
        bytes: &mut [u8; abi::MAX_MESSAGE],
    ) -> Result<(usize, Option<Self>), Error> {
        Self::receive(endpoint, bytes, abi::NOTIFY_TRY_RECEIVE)
    }
    fn receive(
        endpoint: Handle,
        bytes: &mut [u8; abi::MAX_MESSAGE],
        operation: u64,
    ) -> Result<(usize, Option<Self>), Error> {
        let mut packet = Record([0; abi::IPC_PACKET_BYTES]);
        let length = result(arch::call3(
            operation,
            endpoint.raw(),
            packet.0.as_mut_ptr() as u64,
            0,
        ))? as usize;
        if length > abi::MAX_MESSAGE || word(&packet.0[..8]) != length as u64 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        bytes[..length].copy_from_slice(&packet.0[16..16 + length]);
        let handle = word(&packet.0[8..16]);
        Ok((length, (handle != 0).then_some(Self(handle))))
    }
}
