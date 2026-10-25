//! Setup only. Typed acceptance cannot mistake changing bytes for frozen images.
use super::*;
use crate::ipc::Handle;
#[repr(align(128))]
struct Packet([u8; abi::IPC_PACKET_BYTES]);
impl Writer {
    pub fn delegate(&self, endpoint: Handle) -> Result<(), Error> {
        delegate(self.handle(), endpoint)
    }
}
impl Reader {
    pub fn delegate(&self, endpoint: Handle) -> Result<(), Error> {
        delegate(self.handle(), endpoint)
    }
    pub fn accept(endpoint: Handle) -> Result<Self, Error> {
        let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
        raw::result(arch::call(
            abi::IPC_RECEIVE_SHARED,
            endpoint.raw(),
            packet.0.as_mut_ptr() as u64,
        ))?;
        let handle = u64::from_le_bytes(packet.0[8..16].try_into().unwrap());
        if handle == 0 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        Self::received(handle)
    }
}
fn delegate(handle: u64, endpoint: Handle) -> Result<(), Error> {
    let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
    packet.0[8..16].copy_from_slice(&handle.to_le_bytes());
    raw::result(arch::call(
        abi::IPC_SEND_SHARED,
        endpoint.raw(),
        packet.0.as_ptr() as u64,
    ))
    .map(|_| ())
}
