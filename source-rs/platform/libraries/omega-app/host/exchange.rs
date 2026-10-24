//! One request outstanding. Sealed outgoing backing is retained until completion;
//! incoming backing is currently unsupported and explicitly refused.
use super::Host;
use crate::{PACKET_WORDS, PAYLOAD_BYTES, PAYLOAD_WORD};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{matrix as wire, storage};
use cathedral_user_runtime::{
    Error,
    matrix::upstream,
    memory::{Object, Private},
};

pub(super) fn payload(packet: &[u64; PACKET_WORDS]) -> Result<&[u8], Error> {
    let length = usize::try_from(packet[8])
        .ok()
        .filter(|&n| n <= PAYLOAD_BYTES)
        .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
    // SAFETY: u64 contains no padding; the checked byte count stays in this
    // immutable packet borrow. The native ABI is little-endian x86-64.
    Ok(unsafe { core::slice::from_raw_parts(packet[PAYLOAD_WORD..].as_ptr().cast(), length) })
}
fn attachment(packet: &[u64; PACKET_WORDS]) -> Result<Option<Object>, Error> {
    let bytes = payload(packet)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    let mut pages = Private::allocate(bytes.len().div_ceil(4096))?;
    pages.bytes_mut()[..bytes.len()].copy_from_slice(bytes);
    let mut object = pages.freeze()?;
    object.unmap()?;
    Ok(Some(object))
}
pub(super) fn exchange(
    host: &mut Host,
    operation: u64,
    packet: &mut [u64; PACKET_WORDS],
) -> Result<(), Error> {
    let words: wire::Packet = packet[..8].try_into().unwrap();
    let object = attachment(packet)?;
    packet[8] = 0;
    let reply = if operation == 5 {
        let pair = host
            .interaction
            .as_ref()
            .ok_or(Error(abi::DENIED as i64))?
            .pair;
        upstream::exchange(pair, words, object.as_ref())?
    } else {
        let scope = host.storage.as_mut().ok_or(Error(abi::DENIED as i64))?;
        let request = storage::Request::decode(&wire::encode(words))
            .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        #[cfg(feature = "trace-storage")]
        cathedral_user_runtime::time::storage_trace("omega-send", 0, request.op);
        let result = scope.request(request, object.as_ref());
        #[cfg(feature = "trace-storage")]
        cathedral_user_runtime::time::storage_trace("omega-reply", 0, request.op);
        let (bytes, incoming) = result.map_err(|status| Error(status as i64))?;
        if incoming.is_some() {
            // This first binding has no incoming-object operation. Refuse rather
            // than silently dropping a successful reply's attachment contract.
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        wire::decode(&bytes).ok_or(Error(abi::IO_ERROR as i64))?
    };
    packet[..8].copy_from_slice(&reply);
    Ok(())
}
