//! Root-only provider connection: exact shared replies and owner-driven replacement.
use super::*;
use cathedral_user_runtime::{connection::Client, event::Interest, link::Link};
pub(in crate::service::parent) struct Provider {
    link: Link,
    client: Option<Client>,
}
impl Provider {
    pub fn at(index: u64) -> Result<Self, Error> {
        Ok(Self {
            link: Link::at(index)?,
            client: None,
        })
    }
    pub fn client(&mut self, deadline: u64) -> Result<&mut Client, Error> {
        if self.client.is_none() {
            let pair = self.link.connect()?;
            self.adopt(pair, deadline)?;
        }
        Ok(self.client.as_mut().unwrap())
    }
    fn adopt(&mut self, pair: (Handle, Handle), deadline: u64) -> Result<(), Error> {
        self.client = None;
        self.client = Some(Client::connect(pair, deadline)?);
        cathedral_user_runtime::write(b"Cathedral matrix: shared storage connection ready\n")?;
        Ok(())
    }
    /// A still-live accepted link returns BUSY; only an owner grant asks for a
    /// new provider incarnation. Failed/ambiguous operations never replay here.
    pub fn refresh(&mut self) -> Result<bool, Error> {
        match self.link.connect() {
            Ok(pair) => {
                self.adopt(pair, time::after(300)?)?;
                Ok(true)
            }
            Err(Error(c)) if c == abi::BUSY as i64 && self.client.is_some() => Ok(false),
            Err(error) => Err(error),
        }
    }
    /// The non-reused receive name identifies this local incarnation. The
    /// second word is reserved to preserve the owner's opaque recovery key shape.
    pub fn identity(&self) -> Option<[u64; 2]> {
        let Interest::Ipc(receive) = self.client.as_ref()?.interests().ok()?[0] else {
            return None;
        };
        Some([receive.raw(), 0])
    }
    pub fn begin(
        &mut self,
        words: wire::Packet,
        object: Option<&Object>,
        deadline: u64,
    ) -> Result<Handle, Error> {
        let client = self.client(deadline)?;
        client.begin(&wire::encode(words), object)?;
        let Interest::Ipc(receive) = client.interests()?[0] else {
            unreachable!();
        };
        Ok(receive)
    }
    pub fn poll(&mut self, deadline: u64) -> Result<Option<(wire::Packet, Option<Object>)>, Error> {
        let client = self.client.as_mut().ok_or(Error(abi::PEER_CLOSED as i64))?;
        if let Some((bytes, len, object)) = client.poll()? {
            return wire::decode(&bytes[..len])
                .map(|w| Some((w, object)))
                .ok_or(Error(abi::IO_ERROR as i64));
        }
        if time::reached(time::now()?, deadline) {
            return Err(Error(abi::TIMED_OUT as i64));
        }
        Ok(None)
    }
    pub fn wake(&self) -> Option<Interest> {
        let client = self.client.as_ref()?;
        client
            .pending()
            .then(|| client.interests().ok().map(|i| i[1]))
            .flatten()
    }
    pub fn deadline(&self) -> Option<u64> {
        self.client.as_ref()?.transfer_deadline()
    }
    pub fn invalidate(&mut self) {
        self.client = None;
    }
}
