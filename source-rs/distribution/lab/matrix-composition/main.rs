#![no_std]
#![no_main]
//! Boot a separate matrix and three ordinary peers; probe only their IPC boundaries.
//! The `omega-matrix` feature talks the shared v4 protocol the standalone Omega
//! matrix speaks; without it the fixtures use copied requests to the Rust matrix.
#[cfg(feature = "upstream-proof")]
mod nested;
mod peer;
mod proof;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{ipc::Handle, time};
type Pair = (Handle, Handle);
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, argument: u64, environment: cathedral_user_runtime::task::Environment) -> u64 {
    #[cfg(feature = "upstream-proof")]
    if role == 0 {
        return nested::run();
    } else if role == 2 {
        return nested::parent::run();
    }
    if role == 0 {
        proof::run()
    } else {
        peer::run(environment, argument)
    }
}

/// The owner's edge to a matrix's control pair, in whichever protocol it speaks.
pub struct Owner {
    #[cfg(feature = "omega-matrix")]
    client: cathedral_user_runtime::connection::Client,
    #[cfg(not(feature = "omega-matrix"))]
    pair: Pair,
}
impl Owner {
    pub fn connect(pair: Pair) -> Self {
        #[cfg(feature = "omega-matrix")]
        {
            Self {
                client: cathedral_user_runtime::connection::Client::connect(
                    pair,
                    time::after(300).unwrap(),
                )
                .unwrap(),
            }
        }
        #[cfg(not(feature = "omega-matrix"))]
        {
            Self { pair }
        }
    }
    pub fn call(&mut self, words: wire::Packet) -> wire::Packet {
        self.try_call(words).unwrap()
    }
    pub fn try_call(
        &mut self,
        words: wire::Packet,
    ) -> Result<wire::Packet, cathedral_user_runtime::Error> {
        #[cfg(feature = "omega-matrix")]
        {
            let (reply, len) = self.client.call(&wire::encode(words), time::after(300)?)?;
            Ok(wire::decode(&reply[..len]).unwrap())
        }
        #[cfg(not(feature = "omega-matrix"))]
        {
            self.pair.0.send(&wire::encode(words))?;
            let mut bytes = [0; 64];
            let len = self.pair.1.receive_until(&mut bytes, time::after(300)?)?;
            Ok(wire::decode(&bytes[..len]).unwrap())
        }
    }
    pub fn close(self) {
        #[cfg(feature = "omega-matrix")]
        {
            let mut client = self.client;
            client.retire();
        }
        #[cfg(not(feature = "omega-matrix"))]
        {
            self.pair.0.close().unwrap();
            self.pair.1.close().unwrap();
        }
    }
}
