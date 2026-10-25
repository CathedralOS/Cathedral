//! Supplied parent storage stream: shared for separate service bundles; copied
//! only for old fixtures that combine storage and interaction on the same pair.
use super::*;
use cathedral_user_runtime::{connection::Client, event::Interest, storage::Scope};
// Parent is statically allocated; keep custody inline without a heap allocation.
#[allow(clippy::large_enum_variant)]
pub(in crate::service::parent) enum Upstream {
    Shared(Client),
    Combined((Handle, Handle)),
}
impl Upstream {
    pub fn connect(pair: (Handle, Handle)) -> Result<Self, Error> {
        let client = Client::connect(pair, time::after(300)?)?;
        cathedral_user_runtime::write(b"Cathedral matrix: shared parent storage ready\n")?;
        Ok(Self::Shared(client))
    }
    pub fn scope(&mut self) -> Scope<'_> {
        match self {
            Self::Shared(client) => Scope::shared(client),
            Self::Combined(pair) => Scope::new(*pair),
        }
    }
    pub fn receiver(&self) -> Result<Handle, Error> {
        match self {
            Self::Shared(client) => match client.interests()?[0] {
                Interest::Ipc(h) => Ok(h),
                _ => unreachable!(),
            },
            Self::Combined(pair) => Ok(pair.1),
        }
    }
    pub fn begin(&mut self, words: wire::Packet, object: Option<&Object>) -> Result<Handle, Error> {
        match self {
            Self::Shared(client) => client.begin(&wire::encode(words), object)?,
            Self::Combined(pair) => match object {
                Some(object) => pair.0.send_memory(&wire::encode(words), object)?,
                None => pair.0.send(&wire::encode(words))?,
            },
        }
        self.receiver()
    }
    pub fn poll(&mut self, deadline: u64) -> Result<Option<(wire::Packet, Option<Object>)>, Error> {
        let Self::Shared(client) = self else {
            return cathedral_user_runtime::matrix::upstream::exchange::poll(
                self.receiver()?,
                deadline,
            );
        };
        if let Some((bytes, n, object)) = client.poll()? {
            return wire::decode(&bytes[..n])
                .map(|words| Some((words, object)))
                .ok_or(Error(abi::IO_ERROR as i64));
        }
        if time::reached(time::now()?, deadline) {
            return Err(Error(abi::TIMED_OUT as i64));
        }
        Ok(None)
    }
    pub fn call(&mut self, words: wire::Packet) -> Result<wire::Packet, Error> {
        let deadline = time::after(100)?;
        match self {
            Self::Shared(client) => {
                let (bytes, n) = client.call(&wire::encode(words), deadline)?;
                wire::decode(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))
            }
            Self::Combined(pair) => {
                pair.0.send(&wire::encode(words))?;
                let mut bytes = [0; 64];
                let n = pair.1.receive_until(&mut bytes, deadline)?;
                wire::decode(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))
            }
        }
    }
    pub fn wake(&self) -> Option<Interest> {
        match self {
            Self::Shared(client) if client.pending() => client.interests().ok().map(|i| i[1]),
            _ => None,
        }
    }
    pub fn deadline(&self) -> Option<u64> {
        match self {
            Self::Shared(c) => c.transfer_deadline(),
            _ => None,
        }
    }
    pub fn close(self) -> Result<(), Error> {
        if let Self::Combined((send, receive)) = self {
            send.close()?;
            receive.close()?;
        }
        Ok(())
    }
}
