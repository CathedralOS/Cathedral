//! Service control and one boot-approved client, with independent connections.
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
    reply: Handle,
}
pub struct Server {
    control: (Handle, Handle),
    link: Option<Link>,
    client: Option<(Handle, Handle)>,
}
impl Server {
    pub fn open() -> Result<Self, Error> {
        let link = match Link::at(0) {
            Ok(link) => Some(link),
            Err(Error(code)) if code == abi::DENIED as i64 => None,
            Err(error) => return Err(error),
        };
        Ok(Self {
            control: (Handle::bootstrap(0)?, Handle::bootstrap(1)?),
            link,
            client: None,
        })
    }
    pub fn next_request(&mut self) -> Result<Request, Error> {
        loop {
            if self.client.is_none()
                && let Some(link) = self.link
            {
                match link.connect() {
                    Ok(pair) => self.client = Some(pair),
                    Err(Error(code))
                        if code == abi::PEER_CLOSED as i64 || code == abi::BUSY as i64 => {}
                    Err(error) => return Err(error),
                }
            }
            let control = if let Some(client) = self.client {
                match ipc::wait_two(self.control.0, client.0, time::after(25)?) {
                    Ok(index) => index == 0,
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => continue,
                    Err(_) => {
                        self.client = None;
                        continue;
                    }
                }
            } else {
                true
            };
            let pair = if control {
                self.control
            } else {
                self.client.unwrap()
            };
            let mut bytes = [0; 64];
            let result = if self.link.is_some() && self.client.is_none() {
                pair.0.receive_until(&mut bytes, time::after(10)?)
            } else {
                pair.0.receive(&mut bytes)
            };
            match result {
                Ok(len) => {
                    return Ok(Request {
                        bytes,
                        len,
                        control,
                        reply: pair.1,
                    });
                }
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(_) if !control => self.client = None,
                Err(error) => return Err(error),
            }
        }
    }
    pub fn reply(&mut self, request: &Request, bytes: &[u8]) -> Result<(), Error> {
        match request.reply.send(bytes) {
            Err(_) if !request.control => {
                self.client = None;
                Ok(())
            }
            result => result,
        }
    }
}
