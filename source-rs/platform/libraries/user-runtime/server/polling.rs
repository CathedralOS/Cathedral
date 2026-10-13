//! Rotate control/client reads fairly, then park on the bounded live-handle set.
use super::{Event, Request, Server};
use crate::{
    Error, abi,
    ipc::{self, Handle},
    time,
};

impl Server {
    pub(super) fn poll(&mut self) -> Result<Option<Event>, Error> {
        // Rotate across control and data so a flooding client cannot monopolize dispatch.
        for offset in 0..=self.count {
            let channel = (self.cursor + offset) % (self.count + 1);
            let Some(pair) = (if channel == 0 {
                Some(self.control)
            } else {
                self.clients[channel - 1]
            }) else {
                continue;
            };
            let mut bytes = [0; 64];
            let received = if self.memory {
                pair.0.try_receive_memory(&mut bytes)
            } else {
                pair.0
                    .receive_until(&mut bytes, time::now()?)
                    .map(|len| (len, None))
            };
            match received {
                Ok((len, object)) => {
                    self.cursor = (channel + 1) % (self.count + 1);
                    let request = Request {
                        bytes,
                        len,
                        control: channel == 0,
                        client: channel.checked_sub(1),
                        incarnation: pair.0.raw(),
                        reply: pair.1,
                    };
                    return Ok(Some(match object {
                        Some(object) => Event::Memory(request, object),
                        None => Event::Request(request),
                    }));
                }
                Err(Error(code))
                    if code == abi::TIMED_OUT as i64 || code == abi::WOULD_BLOCK as i64 => {}
                Err(_) if channel != 0 => {
                    self.clients[channel - 1] = None;
                    return Ok(Some(Event::Disconnected {
                        client: channel - 1,
                        incarnation: pair.0.raw(),
                    }));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    /// True hands an extra-channel/deadline wake back to the caller without consuming it.
    pub(super) fn wait(&self, extra: &[Handle], deadline: Option<u64>) -> Result<bool, Error> {
        // Watch every live channel together. The bounded maintenance timeout
        // retries absent links; quiet connected clients generate no poll traffic.
        let mut handles = [self.control.0; abi::MAX_WAIT_HANDLES];
        let mut count = 1;
        for pair in self.clients.iter().take(self.count).flatten() {
            handles[count] = pair.0;
            count += 1;
        }
        if count + extra.len() > handles.len() {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        handles[count..count + extra.len()].copy_from_slice(extra);
        let until = deadline.unwrap_or(time::after(100)?);
        match ipc::wait_many(&handles[..count + extra.len()], until) {
            Ok(index) if index >= count => return Ok(true),
            Ok(_) => (),
            Err(Error(code)) if code == abi::TIMED_OUT as i64 && deadline.is_some() => {
                return Ok(true);
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            // Extra handles belong to the caller; do not spin on a stale one.
            Err(error) if !extra.is_empty() => return Err(error),
            // The next scan identifies and publishes the exact closed client.
            Err(_) => (),
        }
        Ok(false)
    }
}
