//! IPC_CREATE -> local endpoints; IPC_SEND_CAP -> kernel escrow -> IPC_RECEIVE_CAP.
//! Either peer may be an ordinary application. No service registry or PID lookup.
use super::{Error, Handle, abi, arch, result};
/// Create a one-way channel, returning (send, receive). Both initially belong here.
pub fn channel() -> Result<(Handle, Handle), Error> {
    let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
    result(arch::call3(
        abi::IPC_CREATE,
        packet.0.as_mut_ptr() as u64,
        0,
        0,
    ))?;
    Ok((Handle(word(&packet.0[..8])), Handle(word(&packet.0[8..16]))))
}
impl Handle {
    /// Close this local grant. All copied Rust Handles with these bits become stale.
    pub fn close(self) -> Result<(), Error> {
        result(arch::call(abi::IPC_CLOSE, self.raw(), 0)).map(|_| ())
    }
    /// Move one dynamic endpoint with a message. Success invalidates every local
    /// copy of `endpoint`; failure preserves it. Boot endpoints cannot be moved.
    pub fn send_with(self, bytes: &[u8], endpoint: Handle) -> Result<(), Error> {
        if bytes.len() > abi::MAX_MESSAGE {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
        packet.0[..8].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
        packet.0[8..16].copy_from_slice(&endpoint.raw().to_le_bytes());
        packet.0[16..16 + bytes.len()].copy_from_slice(bytes);
        result(arch::call3(
            abi::IPC_SEND_CAP,
            self.raw(),
            packet.0.as_ptr() as u64,
            0,
        ))
        .map(|_| ())
    }
    /// Explicitly accept an optional endpoint. Ordinary receive returns TOO_SMALL
    /// without consuming a message carrying one. Use wait_many for a timed wait.
    pub fn receive_with(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
    ) -> Result<(usize, Option<Handle>), Error> {
        self.endpoint_packet(bytes, abi::IPC_RECEIVE_CAP)
    }
    /// Nonblocking typed receive; no message or authority is consumed on refusal.
    pub fn try_receive_with(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
    ) -> Result<(usize, Option<Handle>), Error> {
        self.endpoint_packet(bytes, abi::IPC_TRY_RECEIVE_CAP)
    }
    fn endpoint_packet(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
        operation: u64,
    ) -> Result<(usize, Option<Handle>), Error> {
        let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
        let len = result(arch::call3(
            operation,
            self.raw(),
            packet.0.as_mut_ptr() as u64,
            0,
        ))? as usize;
        if len > abi::MAX_MESSAGE || word(&packet.0[..8]) != len as u64 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        bytes[..len].copy_from_slice(&packet.0[16..16 + len]);
        let handle = word(&packet.0[8..16]);
        Ok((len, (handle != 0).then_some(Handle(handle))))
    }
}
fn word(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}
// 128-byte alignment/size keeps the complete 80-byte wire record inside one page.
#[repr(align(128))]
struct Packet([u8; abi::IPC_PACKET_BYTES]);
