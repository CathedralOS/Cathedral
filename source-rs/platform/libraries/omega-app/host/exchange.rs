//! One request outstanding. Sealed outgoing backing is retained until completion;
//! incoming BULK backing is bounded, copied and released before app resumption.
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
pub(super) fn attachment(packet: &[u64; PACKET_WORDS]) -> Result<Option<Object>, Error> {
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
    let object = super::attachments::take(&mut host.staged, packet)?;
    let receive_limit = packet[11];
    packet[8] = 0;
    packet[11] = 0;
    if receive_limit > PAYLOAD_BYTES as u64
        || (receive_limit != 0 && (operation != 4 || words[0] != storage::BULK))
    {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
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
        let trace_scope = scope.realm();
        #[cfg(feature = "trace-storage")]
        cathedral_user_runtime::time::storage_trace("omega-send", trace_scope, request.op);
        let result = scope.request(request, object.as_ref());
        #[cfg(feature = "trace-storage")]
        cathedral_user_runtime::time::storage_trace("omega-reply", trace_scope, request.op);
        let (bytes, incoming) = result.map_err(|status| Error(status as i64))?;
        let reply = wire::decode(&bytes).ok_or(Error(abi::IO_ERROR as i64))?;
        if let Some(mut incoming) = incoming {
            if request.op != storage::BULK || reply[0] != 0 {
                return Err(Error(abi::INVALID_ARGUMENT as i64));
            }
            let length =
                crate::transfer::received(receive_limit, words[4], reply[4], incoming.len())
                    .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
            let bytes = incoming.map()?;
            // SAFETY: The validated length fits the exclusively borrowed packet.
            let destination = unsafe {
                core::slice::from_raw_parts_mut(
                    packet[PAYLOAD_WORD..].as_mut_ptr().cast::<u8>(),
                    length,
                )
            };
            destination.copy_from_slice(&bytes[..length]);
            packet[8] = length as u64;
        } else if request.op == storage::BULK && reply[0] == 0 && reply[4] != 0 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        reply
    };
    packet[..8].copy_from_slice(&reply);
    Ok(())
}
