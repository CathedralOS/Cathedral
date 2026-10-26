//! Two logical owners retain one shared interaction pipeline and its request IDs.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error, connection::Pipeline, event::Interest, ipc::Handle, matrix::upstream::Pair,
    memory::Object, time,
};
use core::cell::{Cell, RefCell};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum Lane {
    Poll,
    Visual,
}
// The process owns this bounded state inline; no allocator or self-reference.
pub(in super::super) struct Connection {
    client: RefCell<Pipeline<2>>,
    owners: Cell<[Option<u64>; 2]>,
}
impl Connection {
    pub fn shared(pair: Pair) -> Result<Self, Error> {
        Self::from_shared(Pipeline::connect(pair, time::after(300)?)?)
    }
    /// Broker acquisition and steady-state calls share one ID sequence/window.
    pub fn from_shared(client: Pipeline<2>) -> Result<Self, Error> {
        cathedral_user_runtime::write(b"Cathedral matrix: shared parent interaction ready\n")?;
        Ok(Self {
            client: RefCell::new(client),
            owners: Cell::new([None; 2]),
        })
    }
    pub fn receiver(&self) -> Result<Handle, Error> {
        match self.client.borrow().interests()?[0] {
            Interest::Ipc(handle) => Ok(handle),
            _ => unreachable!(),
        }
    }
    pub fn wake(&self) -> Result<Option<Interest>, Error> {
        let client = self.client.borrow();
        if client.pending() {
            Ok(Some(client.interests()?[1]))
        } else {
            Ok(None)
        }
    }
    pub fn deadline(&self) -> Option<u64> {
        self.client.borrow().transfer_deadline()
    }
    pub fn call(
        &self,
        words: wire::Packet,
        object: Option<&Object>,
    ) -> Result<wire::Packet, Error> {
        let reply = self
            .client
            .borrow_mut()
            .call_retained_memory(&wire::encode(words), object)?;
        self.decode(reply)
    }
    pub fn begin(
        &self,
        lane: Lane,
        words: wire::Packet,
        object: Option<&Object>,
    ) -> Result<(), Error> {
        let mut ids = self.owners.get();
        let slot = usize::from(lane == Lane::Visual);
        if ids[slot].is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        let id = self
            .client
            .borrow_mut()
            .submit(&wire::encode(words), object)?;
        ids[slot] = Some(id);
        self.owners.set(ids);
        Ok(())
    }
    pub fn poll(&self) -> Result<Option<(Lane, wire::Packet)>, Error> {
        let Some((id, reply)) = self.client.borrow_mut().poll_reply()? else {
            return Ok(None);
        };
        let mut ids = self.owners.get();
        let Some(slot) = ids.iter().position(|pending| *pending == Some(id)) else {
            self.client.borrow_mut().retire();
            return Err(Error(abi::IO_ERROR as i64));
        };
        ids[slot] = None;
        self.owners.set(ids);
        Ok(Some((
            if slot == 0 { Lane::Poll } else { Lane::Visual },
            self.decode(reply)?,
        )))
    }
    fn decode(
        &self,
        (bytes, len, object): cathedral_user_runtime::connection::Response,
    ) -> Result<wire::Packet, Error> {
        if object.is_none()
            && let Some(words) = wire::decode(&bytes[..len])
        {
            return Ok(words);
        }
        self.client.borrow_mut().retire();
        Err(Error(abi::IO_ERROR as i64))
    }
}
