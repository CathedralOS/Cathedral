//! Lab-owned input ingress. Retain shared correlation throughout fault injection;
//! this connection is retired with the disposable tree, never handed to a driver.
use crate::{Pair, abi, time, wire};
use cathedral_user_runtime::connection::Client;
pub(super) struct Source(Client);
impl Source {
    pub fn open(pair: Pair) -> Self {
        let mut source = Self(Client::connect(pair, time::after(300).unwrap()).unwrap());
        for op in [wire::SPAWN, wire::READ_PIXEL, wire::BIND] {
            assert_eq!(
                source.request([op, 0, 0, 0, 0, 0, 0, 0]),
                wire::error(abi::DENIED)
            );
        }
        cathedral_user_runtime::write(
            b"Cathedral source probe: shared ingress; owner operations denied\n",
        )
        .unwrap();
        source
    }
    pub fn request(&mut self, words: wire::Packet) -> wire::Packet {
        let (bytes, n) = self.0.call_retained(&wire::encode(words)).unwrap();
        wire::decode(&bytes[..n]).expect("source probe reply")
    }
}
