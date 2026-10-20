// Synchronous interaction exchanges. A caller must retire the connection on
// transport failure: without correlation IDs a late reply cannot be reused.
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
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(100)?)
        .inspect_err(|error| {
            time::storage_sample("matrix-exchange-failed", &[words[0], error.0 as u64]);
        })?;
    wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
}
