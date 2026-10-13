//! Typed sealed-memory attachments on ordinary channels. The source keeps its
//! read right; the receiver gains a fresh local, initially unmapped capability.
use super::{Error, Handle, abi, arch, result};
use crate::memory::Object;
#[repr(align(128))]
struct Packet([u8; abi::IPC_PACKET_BYTES]);
impl Handle {
    pub fn send_memory(self, bytes: &[u8], object: &Object) -> Result<(), Error> {
        if bytes.len() > abi::MAX_MESSAGE {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
        packet.0[..8].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
        packet.0[8..16].copy_from_slice(&object.handle().to_le_bytes());
        packet.0[16..16 + bytes.len()].copy_from_slice(bytes);
        result(arch::call(
            abi::IPC_SEND_MEMORY,
            self.raw(),
            packet.0.as_ptr() as u64,
        ))
        .map(|_| ())
    }
    pub fn receive_memory(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
    ) -> Result<(usize, Option<Object>), Error> {
        self.memory_packet(bytes, abi::IPC_RECEIVE_MEMORY)
    }
    pub fn try_receive_memory(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
    ) -> Result<(usize, Option<Object>), Error> {
        self.memory_packet(bytes, abi::IPC_TRY_RECEIVE_MEMORY)
    }
    fn memory_packet(
        self,
        bytes: &mut [u8; abi::MAX_MESSAGE],
        call: u64,
    ) -> Result<(usize, Option<Object>), Error> {
        let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
        let len = result(arch::call(call, self.raw(), packet.0.as_mut_ptr() as u64))? as usize;
        if len > abi::MAX_MESSAGE {
            return Err(Error(abi::IO_ERROR as i64));
        }
        let name = u64::from_le_bytes(packet.0[8..16].try_into().unwrap());
        let object = if name == 0 {
            None
        } else {
            Some(Object::received(name)?)
        };
        bytes[..len].copy_from_slice(&packet.0[16..16 + len]);
        Ok((len, object))
    }
}
