// Root adapter supplies a grant; nested instances obtain the same shape via IPC.
// A lost/invalid upstream grant empties the world before any child can query it.
use super::*;
use cathedral_user_runtime::{ipc::Handle, task::Environment, time};
pub(in crate::service) use storage::{LaunchRequest, Launched};
pub(super) struct Parent {
    recovery: recovery::Recovery,
    pending: Option<presentation::Pending>,
    storage_pending: Option<storage::Pending>,
    scopes: cathedral_matrix::world::storage::Scopes<CHILDREN>,
    storage_links: [Option<storage::Provider>; 2],
    storage_epoch: u64,
    /// The object the upstream binding is rooted at; children without a
    /// scope of their own inherit it as their realm.
    storage_realm: u64,
    storage_failed: u64,
    pub batches: [cathedral_matrix::world::presentation::Batch<
        Option<cathedral_user_runtime::memory::Object>,
    >; CHILDREN],
    next_revision: u64,
    pub upstream: bool,
    images: cathedral_user_runtime::matrix::upstream::Images,
    pair: Option<(Handle, Handle)>,
    storage_pair: Option<(Handle, Handle)>,
    pub separate_storage: bool,
    pub grant: Grant,
    display: Option<(Handle, Handle)>,
    pub wake: Option<Handle>,
    /// MORE from the last snapshot; drain it on another service turn.
    pub refresh_due: bool,
    /// Test-only (delay_probe): F4 arrived from upstream; the next close then
    /// counts this matrix's storage as busy, exchange or none; until when it
    /// does; whether it does this turn.
    #[cfg(feature = "interaction-delay")]
    pub hold_storage_requested: bool,
    #[cfg(feature = "interaction-delay")]
    pub hold_storage_armed: bool,
    #[cfg(feature = "interaction-delay")]
    pub hold_storage_until: Option<u64>,
    #[cfg(feature = "interaction-delay")]
    pub hold_storage_active: bool,
    polling: Option<polling::Pending>,
    next_poll: u32,
    input_capacity: [usize; 2],
}
impl Parent {
    pub(in crate::service) fn request_funding(&mut self, delta: [u64; 5]) -> Result<(), Error> {
        if !self.upstream || self.pair.is_none() {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        let response = match self.call([
            wire::FUND,
            delta[0],
            delta[1],
            delta[2],
            delta[3],
            delta[4],
            0,
            0,
        ]) {
            Ok(reply) => reply,
            Err(error) => {
                // A late acknowledgement must never satisfy the next exchange.
                if let Some((send, receive)) = self.pair.take() {
                    let _ = send.close();
                    let _ = receive.close();
                }
                return Err(error);
            }
        };
        if response[0] == 0 {
            Ok(())
        } else {
            Err(Error(response[0] as i64))
        }
    }
    pub(in super::super) fn has_output(&self) -> bool {
        self.pair.is_some() || self.display.is_some()
    }
    pub const fn empty() -> Self {
        Self {
            recovery: recovery::Recovery::empty(),
            pending: None,
            storage_pending: None,
            scopes: cathedral_matrix::world::storage::Scopes::empty(),
            storage_links: [None, None],
            storage_epoch: 0,
            storage_realm: 0,
            storage_failed: 0,
            batches: [const { cathedral_matrix::world::presentation::Batch::empty() }; CHILDREN],
            next_revision: 0,
            images: cathedral_user_runtime::matrix::upstream::Images::empty(),
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
            refresh_due: false,
            #[cfg(feature = "interaction-delay")]
            hold_storage_requested: false,
            #[cfg(feature = "interaction-delay")]
            hold_storage_armed: false,
            #[cfg(feature = "interaction-delay")]
            hold_storage_until: None,
            #[cfg(feature = "interaction-delay")]
            hold_storage_active: false,
            polling: None,
            next_poll: 0,
            input_capacity: [wire::INPUT_STREAMS, wire::INPUT_GROUPS],
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
        if !world.input_ready() {
            return Ok(());
        }
        self.refresh_due = false;
        // Consume one event per service turn so input cannot monopolize dispatch.
        // MORE keeps the next turn runnable without waiting for another hint.
        let reply = match self.call([wire::POLL, 0, 0, 0, 0, 0, 0, 0]) {
            Ok(reply) => reply,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => {
                self.pair = None;
                world.inherit(Grant::default());
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        self.inherit_reply(world, reply);
        Ok(())
    }
    fn inherit_reply(&mut self, world: &mut World, reply: wire::Packet) {
        #[cfg(feature = "trace-input")]
        if wire::poll_event(reply)[2] == wire::KEY
            && wire::poll_event(reply)[4] != cathedral_platform_contracts::input::RELEASE as u64
        {
            time::trace("matrix-key");
        }
        #[cfg(feature = "interaction-delay")]
        {
            let event = wire::poll_event(reply);
            if event[2] == wire::KEY
                && event[3] == cathedral_platform_contracts::input::F4 as u64
                && event[4] == cathedral_platform_contracts::input::PRESS as u64
            {
                self.hold_storage_requested = true;
            }
        }
        self.grant = world.inherit_poll(reply);
        self.refresh_due = self.grant.valid() && reply[5] & wire::MORE != 0;
    }
    fn call(&self, words: wire::Packet) -> Result<wire::Packet, Error> {
        cathedral_user_runtime::matrix::upstream::call(self.pair.unwrap(), words)
    }
}
