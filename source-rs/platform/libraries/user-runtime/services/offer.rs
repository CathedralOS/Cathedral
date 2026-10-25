//! Retain approved rights until handoff: move endpoints, delegate/release the wait right.
use super::*;
use crate::server::{Request, Server};
pub struct Offer {
    granted: u64,
    endpoints: [Option<Handle>; 7],
    wake: Option<crate::notification::Notification>,
}
impl Offer {
    /// Input may be signalled once the child has acquired its wait right.
    /// Coalescing preserves a hint even if it has not started waiting yet.
    pub fn interaction_ready(&self) -> bool {
        self.wake.is_none()
    }
    pub(crate) fn new(
        granted: u64,
        endpoints: [Option<Handle>; 7],
        wake: crate::notification::Notification,
    ) -> Self {
        Self {
            granted,
            endpoints,
            wake: Some(wake),
        }
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
        let reply = matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]);
        let result = if index == 0 {
            let Some(wake) = self.wake else {
                return server.reply(request, &matrix::encode(matrix::error(wire::TAKEN)));
            };
            server.reply_notification(request, &reply, wake)
        } else {
            let Some(endpoint) = self.endpoints[index] else {
                return server.reply(request, &matrix::encode(matrix::error(wire::TAKEN)));
            };
            server.reply_endpoint(request, &reply, endpoint)
        };
        match result {
            Ok(()) => {
                if index == 0 {
                    // Delegation retained this source right; only receiver escrow
                    // owns it now. The owner separately retains signal/control.
                    self.wake.take().unwrap().release()?;
                } else {
                    self.endpoints[index] = None;
                }
            }
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
        if let Some(wake) = self.wake {
            wake.release()?;
        }
        for endpoint in self.endpoints.into_iter().flatten() {
            endpoint.close()?;
        }
        Ok(())
    }
}
