//! Hold only the already-approved endpoints; successful transfer consumes each once.
use super::*;
use crate::server::{Request, Server};
pub struct Offer {
    granted: u64,
    endpoints: [Option<Handle>; 7],
}
impl Offer {
    /// Keep notification state pending until its receiver has left the broker.
    /// Kernel endpoint transfer requires an empty receive queue.
    pub fn interaction_ready(&self) -> bool {
        self.endpoints[0].is_none()
    }
    pub(crate) fn new(granted: u64, endpoints: [Option<Handle>; 7]) -> Self {
        Self { granted, endpoints }
    }
    pub fn select(&self, words: matrix::Packet) -> Result<usize, u64> {
        wire::select(self.granted, words)
    }
    pub fn producer_ready(&self) -> bool {
        self.granted & imports::INPUT_PRODUCER != 0
            && self.endpoints[3..5].iter().all(Option::is_none)
    }
    pub fn capture_ready(&self) -> bool {
        self.granted & imports::CAPTURE != 0 && self.endpoints[5..7].iter().all(Option::is_none)
    }
    pub fn close_producer(&mut self) -> Result<(), Error> {
        self.granted &= !imports::INPUT_PRODUCER;
        for endpoint in &mut self.endpoints[3..5] {
            if let Some(endpoint) = endpoint.take() {
                endpoint.close()?;
            }
        }
        Ok(())
    }
    /// Caller authenticates the child and incarnation before selecting this offer.
    pub fn reply(
        &mut self,
        server: &mut Server,
        request: &Request,
        words: matrix::Packet,
    ) -> Result<(), Error> {
        let index = match wire::select(self.granted, words) {
            Ok(index) => index,
            Err(code) => return server.reply(request, &matrix::encode(matrix::error(code))),
        };
        let Some(endpoint) = self.endpoints[index] else {
            return server.reply(request, &matrix::encode(matrix::error(wire::TAKEN)));
        };
        match server.reply_endpoint(request, &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]), endpoint) {
            Ok(()) => self.endpoints[index] = None,
            // Transfer failure retains the right. Never mint a replacement or kill
            // the matrix for a child's full queue/death; lifecycle collects leftovers.
            Err(Error(code))
                if code == abi::WOULD_BLOCK as i64 || code == abi::PEER_CLOSED as i64 => {}
            Err(Error(code)) if code == abi::NO_MEMORY as i64 || code == abi::BUSY as i64 => {
                server.reply(request, &matrix::encode(matrix::error(code as u64)))?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }
    pub fn close(self) -> Result<(), Error> {
        for endpoint in self.endpoints.into_iter().flatten() {
            endpoint.close()?;
        }
        Ok(())
    }
}
