// Serialized interaction exchanges keep their one request outstanding across
// slow replies. No timeout retry: a mutation may already have happened.
use crate::{Error, abi, ipc::Handle, memory::Object, time};
use cathedral_platform_contracts::matrix as wire;
pub type Pair = (Handle, Handle);
pub fn call(pair: Pair, words: wire::Packet) -> Result<wire::Packet, Error> {
    exchange(pair, words, None)
}
pub fn exchange(
    pair: Pair,
    words: wire::Packet,
    image: Option<&Object>,
) -> Result<wire::Packet, Error> {
    match image {
        Some(object) => pair.0.send_memory(&wire::encode(words), object)?,
        None => pair.0.send(&wire::encode(words))?,
    }
    let mut bytes = [0; 64];
    let len = receive(pair.1, &mut bytes, words[0], 100).inspect_err(|error| {
        time::storage_sample("matrix-exchange-failed", &[words[0], error.0 as u64]);
    })?;
    wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
}

/// A slow response is not connection loss. Keep custody of this exact request
/// until its reply or a terminal transport error; never send a second request
/// on this pair meanwhile. The threshold is diagnostic, not cancellation.
pub fn receive(
    handle: Handle,
    bytes: &mut [u8; 64],
    operation: u64,
    threshold: u64,
) -> Result<usize, Error> {
    match handle.receive_until(bytes, time::after(threshold)?) {
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
            time::storage_sample("interaction-delayed", &[operation]);
            let result = handle.receive(bytes);
            if result.is_ok() {
                time::storage_sample("interaction-resumed", &[operation]);
            }
            result
        }
        result => result,
    }
}
