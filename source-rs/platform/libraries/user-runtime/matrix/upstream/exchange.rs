//! Nonblocking packet/attachment receive with one absolute deadline. The owner
//! must retire the stream on error: a late reply has no correlation identifier.
use crate::{Error, abi, ipc::Handle, memory::Object, time};
use cathedral_platform_contracts::matrix as wire;
pub fn poll(
    receive: Handle,
    deadline: u64,
) -> Result<Option<(wire::Packet, Option<Object>)>, Error> {
    let mut bytes = [0; 64];
    match receive.try_receive_memory(&mut bytes) {
        Ok((len, object)) => wire::decode(&bytes[..len])
            .map(|words| Some((words, object)))
            .ok_or(Error(abi::IO_ERROR as i64)),
        Err(Error(code)) if code == abi::WOULD_BLOCK as i64 || code == abi::TIMED_OUT as i64 => {
            if time::reached(time::now()?, deadline) {
                Err(Error(abi::TIMED_OUT as i64))
            } else {
                Ok(None)
            }
        }
        Err(error) => Err(error),
    }
}
