// Root adapter supplies a grant; nested instances obtain the same shape via IPC.
// A lost/invalid upstream grant empties the world before any child can query it.
use super::*;
use cathedral_user_runtime::{ipc::Handle, task::Environment, time};
pub(super) struct Parent {
    pending: Option<presentation::Pending>,
    storage_pending: Option<storage::Pending>,
    scopes: cathedral_matrix_service::world::storage::Scopes,
    storage_links: [Option<storage::Provider>; 2],
    storage_epoch: u64,
    storage_failed: u64,
    pub batches: [cathedral_matrix_service::world::presentation::Batch<
        Option<cathedral_user_runtime::memory::Object>,
    >; 2],
    next_revision: u64,
    pub upstream: bool,
    images: [Option<(u64, cathedral_user_runtime::memory::Object)>; wire::MAX_SURFACES],
    pair: Option<(Handle, Handle)>,
    storage_pair: Option<(Handle, Handle)>,
    pub separate_storage: bool,
    pub grant: Grant,
    display: Option<(Handle, Handle)>,
    pub wake: Option<Handle>,
}
impl Parent {
    pub(in super::super) fn has_output(&self) -> bool {
        self.pair.is_some() || self.display.is_some()
    }
    pub fn refresh_notified(&mut self, world: &mut World) -> Result<(), Error> {
        if let Some(wake) = self.wake {
            match wake.receive_until(&mut [0; 64], time::now()?) {
                Ok(_) => self.refresh(world)?,
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    pub const fn empty() -> Self {
        Self {
            pending: None,
            storage_pending: None,
            scopes: cathedral_matrix_service::world::storage::Scopes::empty(),
            storage_links: [None, None],
            storage_epoch: 0,
            storage_failed: 0,
            batches: [const { cathedral_matrix_service::world::presentation::Batch::empty() }; 2],
            next_revision: 0,
            images: [const { None }; wire::MAX_SURFACES],
            upstream: false,
            pair: None,
            storage_pair: None,
            separate_storage: false,
            grant: Grant {
                epoch: 0,
                width: 0,
                height: 0,
                keys: 0,
            },
            display: None,
            wake: None,
        }
    }
    pub fn initialize(
        &mut self,
        environment: &Environment,
        storage_root: bool,
    ) -> Result<(), Error> {
        if (storage_root && !matches!(environment.len(), 0 | 4))
            || !matches!(environment.len(), 0 | 2 | 3 | 4 | 5)
        {
            return Err(Error(abi::DENIED as i64));
        }
        if storage_root {
            self.storage_links[0] = Some(storage::Provider::at(0)?);
            self.storage_links[1] = Some(storage::Provider::at(1)?);
        }
        self.separate_storage = environment.len() == 5 || (storage_root && environment.len() == 4);
        self.upstream = matches!(environment.len(), 2 | 3 | 5);
        let pair = || {
            (
                environment.endpoint(0).unwrap(),
                environment.endpoint(1).unwrap(),
            )
        };
        self.pair = self.upstream.then(pair);
        self.display = (environment.len() == 4).then(pair);
        self.wake = (environment.len() == 3).then(|| environment.endpoint(2).unwrap());
        if environment.len() == 5 {
            let connections = cathedral_user_runtime::matrix::Connections::load(environment)?;
            self.pair = Some(connections.interaction);
            self.storage_pair = Some(connections.storage);
            self.wake = Some(connections.wake);
        } else {
            // Manually assembled headless fixtures retain their original shared stream.
            self.storage_pair = self.pair;
        }
        Ok(())
    }
    pub fn refresh(&mut self, world: &mut World) -> Result<(), Error> {
        if !self.upstream {
            world.inherit(self.grant);
            return Ok(());
        }
        if self.pair.is_none() {
            world.inherit(Grant::default());
            return Ok(());
        }
        // Each round trip returns a consistent local grant and event. MORE avoids
        // spending another scheduling turn merely to discover an empty queue.
        for _ in 0..8 {
            let reply = match self.call([wire::POLL, 0, 0, 0, 0, 0, 0, 0]) {
                Ok(reply) => reply,
                Err(Error(code)) if code == abi::PEER_CLOSED as i64 => {
                    self.pair = None;
                    world.inherit(Grant::default());
                    return Ok(());
                }
                Err(error) => return Err(error),
            };
            self.grant = if reply[0] == 0 {
                Grant {
                    epoch: reply[1],
                    width: reply[2],
                    height: reply[3],
                    keys: reply[4] & wire::VALID_KEY_MASK,
                }
            } else {
                Grant::default()
            };
            world.inherit(self.grant);
            world.surface_capacity((reply[4] >> wire::SURFACE_BITS) as usize);
            if !self.grant.valid() {
                return Ok(());
            }
            let event = wire::poll_event(reply);
            match event[2] {
                wire::NONE => (),
                wire::POINTER => world.pointer(event[3], event[4], event[5]),
                wire::KEY => {
                    #[cfg(feature = "trace-input")]
                    time::trace("matrix-key");
                    world.key(event[3], event[4]);
                }
                wire::CANCEL => world.cancel(),
                _ => {
                    world.inherit(Grant::default());
                    break;
                }
            }
            if reply[5] & wire::MORE == 0 {
                break;
            }
        }
        Ok(())
    }
    fn call(&self, words: wire::Packet) -> Result<wire::Packet, Error> {
        let (send, receive) = self.pair.unwrap();
        send.send(&wire::encode(words))?;
        let mut bytes = [0; 64];
        let len = receive.receive_until(&mut bytes, time::after(100)?)?;
        wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
    }
}
