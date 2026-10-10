//! Fair, bounded service control and up to two boot-approved client connections.
//! Client identity comes from the accepted link slot, never from message contents.
use crate::{
    Error, abi,
    ipc::{self, Handle},
    link::Link,
    time,
};
pub struct Request {
    pub bytes: [u8; 64],
    pub len: usize,
    pub control: bool,
    pub client: Option<usize>,
    pub incarnation: u64,
    reply: Handle,
}
pub enum Event {
    Request(Request),
    Disconnected { client: usize, incarnation: u64 },
}
pub struct Server {
    control: (Handle, Handle),
    links: [Option<Link>; 2],
    clients: [Option<(Handle, Handle)>; 2],
    count: usize,
    cursor: usize,
    idle: usize,
    closed: [Option<u64>; 2],
}
impl Server {
    pub fn open() -> Result<Self, Error> {
        Self::open_clients(1)
    }
    pub fn open_clients(count: usize) -> Result<Self, Error> {
        if count == 0 || count > 2 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut links = [None; 2];
        for (index, slot) in links.iter_mut().enumerate().take(count) {
            *slot = match Link::at(index as u64) {
                Ok(link) => Some(link),
                Err(Error(code)) if code == abi::DENIED as i64 => None,
                Err(error) => return Err(error),
            };
        }
        Ok(Self {
            control: (Handle::bootstrap(0)?, Handle::bootstrap(1)?),
            links,
            clients: [None; 2],
            count,
            cursor: 0,
            idle: 0,
            closed: [None; 2],
        })
    }
    pub fn next_request(&mut self) -> Result<Request, Error> {
        loop {
            if let Event::Request(request) = self.next_event()? {
                return Ok(request);
            }
        }
    }
    /// Services retaining client-owned state must observe disconnects before reuse.
    pub fn next_event(&mut self) -> Result<Event, Error> {
        // A control-only service needs no clock grant (for example the display fixture).
        if self.links.iter().all(Option::is_none) {
            let mut bytes = [0; 64];
            let len = self.control.0.receive(&mut bytes)?;
            return Ok(Event::Request(Request {
                bytes,
                len,
                control: true,
                client: None,
                incarnation: self.control.0.raw(),
                reply: self.control.1,
            }));
        }
        loop {
            for client in 0..self.count {
                if let Some(incarnation) = self.closed[client].take() {
                    return Ok(Event::Disconnected {
                        client,
                        incarnation,
                    });
                }
            }
            self.connect()?;
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
                match pair.0.receive_until(&mut bytes, time::now()?) {
                    Ok(len) => {
                        self.cursor = (channel + 1) % (self.count + 1);
                        return Ok(Event::Request(Request {
                            bytes,
                            len,
                            control: channel == 0,
                            client: channel.checked_sub(1),
                            incarnation: pair.0.raw(),
                            reply: pair.1,
                        }));
                    }
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                    Err(_) if channel != 0 => {
                        self.clients[channel - 1] = None;
                        return Ok(Event::Disconnected {
                            client: channel - 1,
                            incarnation: pair.0.raw(),
                        });
                    }
                    Err(error) => return Err(error),
                }
            }
            // Existing wait-two ABI bounds idle latency for the third channel to one tick.
            self.idle = (self.idle + 1) % self.count;
            let peer = self.clients[self.idle].map_or(self.control.0, |pair| pair.0);
            match ipc::wait_two(self.control.0, peer, time::after(1)?) {
                Ok(_) => (),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(_) => {
                    self.disconnect(self.idle);
                }
            }
        }
    }
    fn disconnect(&mut self, client: usize) {
        if let Some(pair) = self.clients[client].take() {
            self.closed[client] = Some(pair.0.raw());
        }
    }
    fn connect(&mut self) -> Result<(), Error> {
        for index in 0..self.count {
            if self.clients[index].is_none()
                && let Some(link) = self.links[index]
            {
                match link.connect() {
                    Ok(pair) => self.clients[index] = Some(pair),
                    Err(Error(code))
                        if code == abi::PEER_CLOSED as i64 || code == abi::BUSY as i64 => {}
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(())
    }
    pub fn reply(&mut self, request: &Request, bytes: &[u8]) -> Result<(), Error> {
        match request.reply.send(bytes) {
            // A full client response queue loses only that reply. The service and other
            // client stay live; synchronous clients reconcile ambiguous commits.
            Err(Error(code)) if !request.control && code == abi::WOULD_BLOCK as i64 => Ok(()),
            Err(_) if !request.control => {
                self.disconnect(request.client.unwrap());
                Ok(())
            }
            result => result,
        }
    }
}
