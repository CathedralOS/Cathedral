//! Pre-driver fault probes inject through the untouched source pair. The driver
//! later takes this pair and opens shared transport; no shared client is aliased.
use crate::{Pair, time, wire};
pub(super) fn request(pair: Pair, words: wire::Packet) -> wire::Packet {
    pair.0.send(&wire::encode(words)).unwrap();
    let mut bytes = [0; 64];
    let n = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    wire::decode(&bytes[..n]).expect("source probe reply")
}
