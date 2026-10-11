//! Fair, bounded service control and boot-approved client connections.
//! Client identity comes from the accepted link slot, never from message contents.
use crate::{
    Error, abi,
    ipc::{self, Handle},
    link::Link,
    time,
};
#[derive(Clone, Copy)]
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
const MAX_CLIENTS: usize = abi::MAX_WAIT_HANDLES - 1; // Control also occupies a wait slot.
pub struct Server {
    control: (Handle, Handle),
    links: [Option<Link>; MAX_CLIENTS],
    clients: [Option<(Handle, Handle)>; MAX_CLIENTS],
    count: usize,
    cursor: usize,
    closed: [Option<u64>; MAX_CLIENTS],
}
impl Server {
    pub fn open() -> Result<Self, Error> {
        Self::open_clients(1)
    }
    pub fn open_clients(count: usize) -> Result<Self, Error> {
        if count == 0 || count > MAX_CLIENTS {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut links = [None; MAX_CLIENTS];
        for (index, slot) in links.iter_mut().enumerate().take(count) {
            *slot = match Link::at(index as u64) {
                Ok(link) => Some(link),
                // No links is DENIED; an index beyond existing links is BAD_HANDLE.
                // Both represent an absent optional connection in this fixed budget.
                Err(Error(code))
                    if code == abi::DENIED as i64 || code == abi::BAD_HANDLE as i64 =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
        }
        Ok(Self {
            control: (Handle::bootstrap(0)?, Handle::bootstrap(1)?),
            links,
            clients: [None; MAX_CLIENTS],
            count,
            cursor: 0,
            closed: [None; MAX_CLIENTS],
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
            // Watch every live channel together. The bounded maintenance timeout
            // retries absent links; quiet connected clients generate no poll traffic.
            let mut handles = [self.control.0; abi::MAX_WAIT_HANDLES];
            let mut count = 1;
            for pair in self.clients.iter().take(self.count).flatten() {
                handles[count] = pair.0;
                count += 1;
            }
            match ipc::wait_many(&handles[..count], time::after(100)?) {
                Ok(_) => (),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                // The next scan identifies and publishes the exact closed client.
                Err(_) => (),
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
