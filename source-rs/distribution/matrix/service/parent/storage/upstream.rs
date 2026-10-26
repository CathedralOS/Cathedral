//! One supplied shared storage stream, independent of interaction and control.
use super::*;
use cathedral_user_runtime::{connection::Client, event::Interest, storage::Scope};
pub(in crate::service::parent) struct Upstream(Client);
impl Upstream {
    pub fn connect(pair: (Handle, Handle)) -> Result<Self, Error> {
        let client = Client::connect(pair, time::after(300)?)?;
        cathedral_user_runtime::write(b"Cathedral matrix: shared parent storage ready\n")?;
        Ok(Self(client))
    }
    pub fn scope(&mut self) -> Scope<'_> {
        Scope::shared(&mut self.0)
    }
    pub fn receiver(&self) -> Result<Handle, Error> {
        match self.0.interests()?[0] {
            Interest::Ipc(h) => Ok(h),
            _ => unreachable!(),
        }
    }
    pub fn begin(&mut self, words: wire::Packet, object: Option<&Object>) -> Result<Handle, Error> {
        self.0.begin(&wire::encode(words), object)?;
        self.receiver()
    }
    pub fn poll(&mut self, deadline: u64) -> Result<Option<(wire::Packet, Option<Object>)>, Error> {
        if let Some((bytes, n, object)) = self.0.poll()? {
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
        let (bytes, n) = self.0.call(&wire::encode(words), time::after(100)?)?;
        wire::decode(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))
    }
    pub fn wake(&self) -> Option<Interest> {
        if self.0.pending() {
            self.0.interests().ok().map(|i| i[1])
        } else {
            None
        }
    }
    pub fn deadline(&self) -> Option<u64> {
        self.0.transfer_deadline()
    }
}
