//! Owner-only event reception, with no service-link discovery. Retain one exact
//! request until its reply; losing the link leaves independent input usable.
use super::*;
use crate::{
    event::{self, Interest},
    time,
};

pub struct OwnerControl {
    server: Option<Server>,
    request: Option<Request>,
}
impl OwnerControl {
    pub fn open(slot: &'static mut SharedSlot) -> Result<Self, Error> {
        let mut server = Server::open_clients(0)?;
        server.enable_shared_control(slot)?;
        server.enable_memory();
        Ok(Self {
            server: Some(server),
            request: None,
        })
    }
    /// Complete an explicitly selected shared control connection before expensive
    /// app startup. No application request is consumed: poll advances one setup
    /// step, and readiness is checked before polling the established stream.
    /// Use only when the owner initiates setup independently of app readiness.
    pub fn establish(&mut self, deadline: u64) -> Result<(), Error> {
        if self.request.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        let server = self.server.as_mut().ok_or(Error(abi::PEER_CLOSED as i64))?;
        server.establish_control(deadline)
    }
    pub fn reply(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let request = self.request.take().ok_or(Error(abi::DENIED as i64))?;
        self.server
            .as_mut()
            .ok_or(Error(abi::PEER_CLOSED as i64))?
            .reply(&request, bytes)
    }
    /// None means caller-owned readiness or deadline: consume/recheck that source.
    /// Pending owner requests must be answered before receiving another event.
    pub fn next(
        &mut self,
        extra: Interest,
        deadline: Option<u64>,
    ) -> Result<Option<([u8; 64], usize)>, Error> {
        if self.request.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        loop {
            let result = match &mut self.server {
                Some(server) => server.next_event_for(&[extra], deadline),
                None => event::wait(&[extra], deadline.unwrap_or(time::after(100)?)).map(|_| None),
            };
            match result {
                Ok(Some(Event::Request(request))) => {
                    self.request = Some(request);
                    return Ok(Some((request.bytes, request.len)));
                }
                Ok(Some(Event::Memory(request, object))) => {
                    drop(object);
                    let mut refusal = [0; 64];
                    refusal[..8].copy_from_slice(&abi::INVALID_ARGUMENT.to_le_bytes());
                    self.server.as_mut().unwrap().reply(&request, &refusal)?;
                }
                Ok(Some(Event::Disconnected { .. })) => unreachable!(),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => return Ok(None),
                Err(_) if self.server.is_some() => {
                    // A malformed/failed owner stream is not app death. A bad
                    // independent interest is still reported by the next wait.
                    if let Some(mut server) = self.server.take() {
                        server.reset_shared(0);
                        server.control.0.close().ok();
                        server.control.1.close().ok();
                    }
                }
                Ok(None) => return Ok(None),
                Err(error) => return Err(error),
            }
        }
    }
}
