//! Rotate control/client reads fairly, then park on the bounded live-handle set.
use super::{Event, Request, Server};
use crate::{
    Error, abi,
    event::{self, Interest},
    ipc::Handle,
    time,
};

impl Server {
    pub(super) fn poll(&mut self) -> Result<Option<Event>, Error> {
        // Rotate across control and data so a flooding client cannot monopolize dispatch.
        // One clock read serves the whole scan; a wake has already pointed the
        // cursor at the channel the kernel reported ready.
        let now = if self.memory { 0 } else { time::now()? };
        for offset in 0..=self.count {
            let channel = (self.cursor + offset) % (self.count + 1);
            if (self.paused | self.reply_paused()) & (1 << channel) != 0 {
                continue;
            }
            let Some(pair) = (if channel == 0 {
                Some(self.control)
            } else {
                self.clients[channel - 1]
            }) else {
                continue;
            };
            if channel != 0 && self.shared_active(channel - 1) {
                match self.shared_poll(channel - 1, pair) {
                    Ok(Some(event)) => {
                        self.cursor = (channel + 1) % (self.count + 1);
                        return Ok(Some(event));
                    }
                    Ok(None) => continue,
                    Err(_) => {
                        self.disconnect(channel - 1);
                        return Ok(self.take_disconnection());
                    }
                }
            }
            let mut bytes = [0; 64];
            let received = if self.memory {
                pair.0.try_receive_memory(&mut bytes)
            } else {
                pair.0.receive_until(&mut bytes, now).map(|len| (len, None))
            };
            match received {
                Ok((len, object)) => {
                    self.cursor = (channel + 1) % (self.count + 1);
                    if channel != 0
                        && object.is_none()
                        && self.shared_open(channel - 1, pair, &bytes[..len])
                    {
                        if let Some(event) = self.take_disconnection() {
                            return Ok(Some(event));
                        }
                        continue;
                    }
                    let request = Request {
                        bytes,
                        len,
                        control: channel == 0,
                        client: channel.checked_sub(1),
                        incarnation: pair.0.raw(),
                        reply: pair.1,
                        shared_id: 0,
                    };
                    return Ok(Some(match object {
                        Some(object) => Event::Memory(request, object),
                        None => Event::Request(request),
                    }));
                }
                Err(Error(code))
                    if code == abi::TIMED_OUT as i64 || code == abi::WOULD_BLOCK as i64 => {}
                Err(_) if channel != 0 => {
                    self.disconnect(channel - 1);
                    return Ok(self.take_disconnection());
                }
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    /// True hands an extra-channel/deadline wake back to the caller without
    /// consuming it. A wake on a client channel points the poll cursor at that
    /// channel, so the next scan finds it first instead of asking every quiet
    /// client again.
    pub(super) fn wait(&mut self, extra: &[Handle], deadline: Option<u64>) -> Result<bool, Error> {
        // Watch every live channel together. The bounded maintenance timeout
        // retries absent links; quiet connected clients generate no poll traffic.
        let mut handles = [Interest::Ipc(self.control.0); abi::MAX_WAIT_HANDLES];
        let mut channels = [0; abi::MAX_WAIT_HANDLES];
        let mut count = 0;
        if self.paused & 1 == 0 {
            count = 1;
        }
        for index in 0..self.count {
            if (self.paused | self.reply_paused()) & (1 << (index + 1)) == 0
                && let Some(pair) = self.clients[index]
            {
                handles[count] = Interest::Ipc(pair.0);
                channels[count] = index + 1;
                count += 1;
            }
        }
        if let Some((client, wake)) = self.shared_wake()
            && (self.paused | self.reply_paused()) & (1 << (client + 1)) == 0
        {
            if count == handles.len() {
                return Err(Error(abi::INVALID_ARGUMENT as i64));
            }
            handles[count] = Interest::Notification(wake);
            channels[count] = client + 1;
            count += 1;
        }
        if count + extra.len() > handles.len() {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        for (out, &handle) in handles[count..count + extra.len()].iter_mut().zip(extra) {
            *out = Interest::Ipc(handle);
        }
        let mut until = deadline.unwrap_or(time::after(100)?);
        if let Some(setup) = self.shared_deadline()
            && time::reached(until, setup)
        {
            until = setup;
        }
        if self.reply_paused() != 0 {
            let retry = time::after(1)?;
            if time::reached(until, retry) {
                until = retry;
            }
        }
        if count + extra.len() == 0 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        match event::wait(&handles[..count + extra.len()], until) {
            Ok(index) if index >= count => return Ok(true),
            Ok(index) => self.cursor = channels[index],
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
