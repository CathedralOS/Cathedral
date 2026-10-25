//! Two independent logical owners share one persistent interaction pipeline.
//! Compatibility is selected by composition, never after shared setup fails.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    connection::Pipeline,
    event::Interest,
    ipc::Handle,
    matrix::upstream::{self, Pair},
    memory::Object,
    time,
};
use core::cell::{Cell, RefCell};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum Lane {
    Poll,
    Visual,
}
// The process owns this bounded state inline; no allocator or self-reference.
#[allow(clippy::large_enum_variant)]
pub(in super::super) enum Connection {
    Copied(Pair),
    Shared {
        client: RefCell<Pipeline<2>>,
        owners: Cell<[Option<u64>; 2]>,
    },
}
impl Connection {
    pub fn shared(pair: Pair) -> Result<Self, Error> {
        let client = Pipeline::connect(pair, time::after(300)?)?;
        Self::from_shared(client)
    }
    /// Broker acquisition and steady-state calls share one ID sequence/window.
    pub fn from_shared(client: Pipeline<2>) -> Result<Self, Error> {
        cathedral_user_runtime::write(b"Cathedral matrix: shared parent interaction ready\n")?;
        Ok(Self::Shared {
            client: RefCell::new(client),
            owners: Cell::new([None; 2]),
        })
    }
    pub fn receiver(&self) -> Result<Handle, Error> {
        match self {
            Self::Copied(pair) => Ok(pair.1),
            Self::Shared { client, .. } => match client.borrow().interests()?[0] {
                Interest::Ipc(handle) => Ok(handle),
                _ => unreachable!(),
            },
        }
    }
    pub fn wake(&self) -> Result<Option<Interest>, Error> {
        match self {
            Self::Copied(_) => Ok(None),
            Self::Shared { client, .. } => {
                let client = client.borrow();
                if client.pending() {
                    Ok(Some(client.interests()?[1]))
                } else {
                    Ok(None)
                }
            }
        }
    }
    pub fn deadline(&self) -> Option<u64> {
        match self {
            Self::Copied(_) => None,
            Self::Shared { client, .. } => client.borrow().transfer_deadline(),
        }
    }
    pub fn call(
        &self,
        words: wire::Packet,
        object: Option<&Object>,
    ) -> Result<wire::Packet, Error> {
        match self {
            Self::Copied(pair) => upstream::exchange(*pair, words, object),
            Self::Shared { client, .. } => {
                let reply = client
                    .borrow_mut()
                    .call_retained_memory(&wire::encode(words), object)?;
                Self::decode(client, reply)
            }
        }
    }
    pub fn begin(
        &self,
        lane: Lane,
        words: wire::Packet,
        object: Option<&Object>,
    ) -> Result<(), Error> {
        match self {
            Self::Copied(pair) => match object {
                Some(object) => pair.0.send_memory(&wire::encode(words), object),
                None => pair.0.send(&wire::encode(words)),
            },
            Self::Shared { client, owners } => {
                let mut ids = owners.get();
                let slot = usize::from(lane == Lane::Visual);
                if ids[slot].is_some() {
                    return Err(Error(abi::BUSY as i64));
                }
                let id = client.borrow_mut().submit(&wire::encode(words), object)?;
                ids[slot] = Some(id);
                owners.set(ids);
                Ok(())
            }
        }
    }
    pub fn poll(&self) -> Result<Option<(Option<Lane>, wire::Packet)>, Error> {
        match self {
            Self::Copied(pair) => {
                let mut bytes = [0; 64];
                match pair.1.receive_until(&mut bytes, time::now()?) {
                    Ok(n) => wire::decode(&bytes[..n])
                        .map(|words| Some((None, words)))
                        .ok_or(Error(abi::IO_ERROR as i64)),
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => Ok(None),
                    Err(error) => Err(error),
                }
            }
            Self::Shared { client, owners } => {
                let Some((id, reply)) = client.borrow_mut().poll_reply()? else {
                    return Ok(None);
                };
                let mut ids = owners.get();
                let Some(slot) = ids.iter().position(|pending| *pending == Some(id)) else {
                    client.borrow_mut().retire();
                    return Err(Error(abi::IO_ERROR as i64));
                };
                ids[slot] = None;
                owners.set(ids);
                Ok(Some((
                    Some(if slot == 0 { Lane::Poll } else { Lane::Visual }),
                    Self::decode(client, reply)?,
                )))
            }
        }
    }
    fn decode(
        client: &RefCell<Pipeline<2>>,
        (bytes, len, object): cathedral_user_runtime::connection::Response,
    ) -> Result<wire::Packet, Error> {
        if object.is_none()
            && let Some(words) = wire::decode(&bytes[..len])
        {
            return Ok(words);
        }
        client.borrow_mut().retire();
        Err(Error(abi::IO_ERROR as i64))
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        if let Self::Copied((send, receive)) = self {
            let _ = send.close();
            let _ = receive.close();
        }
    }
}
