// One level of ownership. The same state machine runs above apps or other worlds.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use children::Child;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Grant {
    pub epoch: u64,
    pub width: u64,
    pub height: u64,
    pub keys: u64,
}
impl Grant {
    pub fn valid(self) -> bool {
        self.epoch != 0
            && self.width > 0
            && self.height > 0
            && self.width <= wire::MAX_EXTENT
            && self.height <= wire::MAX_EXTENT
            && self.keys & !wire::VALID_KEY_MASK == 0
    }
}
pub struct World<const N: usize = 2> {
    parent: Grant,
    frames: bool,
    capacity: usize,
    authority: u64,
    retired: [Option<wire::Packet>; wire::MAX_SURFACES],
    children: [Child; N],
    layout_controller: Option<(usize, u64)>,
    input: input::Input,
    producers: [input::Producer; N],
    next_producer: [u64; N],
}
impl<const N: usize> World<N> {
    pub fn binding_matches(&self, slot: usize, incarnation: u64, epoch: u64) -> bool {
        self.children.get(slot).is_some_and(|c| {
            c.incarnation == incarnation && c.grant.valid() && c.grant.epoch == epoch
        })
    }
    /// Retained process state can be installed without a large stack temporary.
    pub const fn empty() -> Self {
        Self {
            parent: Grant {
                epoch: 0,
                width: 0,
                height: 0,
                keys: 0,
            },
            frames: true,
            capacity: 0,
            authority: 0,
            retired: [None; wire::MAX_SURFACES],
            children: [const { Child::empty() }; N],
            layout_controller: None,
            input: input::Input::empty(),
            producers: [input::Producer::empty(); N],
            next_producer: [0; N],
        }
    }

    pub fn inherit(&mut self, grant: Grant) {
        let grant = if grant.valid() {
            grant
        } else {
            Grant::default()
        };
        if self.parent == grant {
            return;
        }
        for slot in 0..N {
            self.revoke_input_producer(slot);
        }
        self.authority = self.authority.checked_add(1).expect("authority generation");
        let replaced = self.parent.epoch != grant.epoch;
        if replaced {
            self.layout_controller = None;
            self.input = input::Input::empty();
        }
        if self.parent.valid() && (replaced || !grant.valid()) {
            // Losing the parent's authority cannot be repaired by silently reviving
            // old descendant grants when a replacement parent later reconnects.
            for slot in 0..N {
                self.retire_surfaces(slot);
                self.children[slot].rect = None;
                self.children[slot].surface_limit = 0;
            }
        }
        self.parent = grant;
        let mut changed = [false; N];
        for (slot, child) in self.children.iter_mut().enumerate() {
            let previous = child.grant;
            child.refresh(grant, replaced);
            child.demand(self.frames && child.visible);
            if child.grant != previous {
                changed[slot] = true;
                self.input.forget(slot);
            }
        }
        for (slot, changed) in changed.into_iter().enumerate() {
            if changed {
                self.retire_surfaces(slot);
            }
        }
    }
    pub fn place(&mut self, slot: usize, rect: Rect, keys: u64) -> Result<(), u64> {
        if !self.parent.valid() || !rect.valid() || keys & !self.parent.keys != 0 {
            return Err(abi::DENIED);
        }
        let child = self.children.get_mut(slot).ok_or(abi::DENIED)?;
        let previous = child.grant;
        child.rect = Some(rect);
        child.keys = keys;
        child.refresh(self.parent, false);
        child.demand(self.frames && child.visible);
        if child.grant != previous {
            self.revoke_input_producer(slot);
            self.retire_surfaces(slot);
            self.input.forget(slot);
        }
        Ok(())
    }
    pub fn revoke(&mut self, slot: usize) -> Result<(), u64> {
        if slot >= self.children.len() {
            return Err(abi::DENIED);
        }
        self.revoke_input_producer(slot);
        if self
            .layout_controller
            .is_some_and(|(controller, _)| controller == slot)
        {
            self.layout_controller = None;
        }
        self.input.forget(slot);
        self.children[slot].rect = None;
        self.retire_surfaces(slot);
        self.children[slot].surface_limit = 0;
        self.children[slot].refresh(self.parent, false);
        self.children[slot].demand(self.frames && self.children[slot].visible);
        Ok(())
    }
    pub fn disconnected(&mut self, slot: usize) {
        let _ = self.revoke(slot);
        if let Some(child) = self.children.get_mut(slot) {
            child.incarnation = 0;
            child.input_capacity = [wire::INPUT_STREAMS, wire::INPUT_GROUPS];
        }
    }
    /// Coalesced wake hint; clients re-read only their local view and events.
    pub fn take_notification(&mut self, slot: usize) -> bool {
        core::mem::take(&mut self.children[slot].notify)
    }
    /// A child-local lifecycle result changed; wake its observer to query it.
    pub fn notify(&mut self, slot: usize) {
        self.children[slot].notify = true;
    }
    /// Identity is supplied by the accepted endpoint, never a request field.
    pub fn request(&mut self, slot: usize, incarnation: u64, words: wire::Packet) -> wire::Packet {
        if slot >= self.children.len() {
            return wire::error(abi::DENIED);
        }
        let previous = self.children[slot].incarnation;
        if previous != 0 && previous != incarnation {
            self.disconnected(slot);
        }
        let child = &mut self.children[slot];
        child.incarnation = incarnation;
        if wire::producer_operation(words[0]) {
            return self.producer_request(slot, incarnation, words);
        }
        match words {
            [wire::CURSOR_PREFERENCE, epoch, channel, mode, 0, 0, 0, 0] => {
                child.cursor_preference(epoch, channel, mode)
            }
            [wire::FRAME_DEMAND, epoch, 0, 0, 0, 0, 0, 0]
                if child.grant.valid() && epoch == child.grant.epoch =>
            {
                [
                    0,
                    epoch,
                    child.frame.pending.map_or(0, |(ticket, _)| ticket),
                    0,
                    0,
                    0,
                    0,
                    0,
                ]
            }
            [wire::FRAME_READY, epoch, ticket, 0, 0, 0, 0, 0]
                if child.grant.valid()
                    && epoch == child.grant.epoch
                    && child.frame.pending.is_some_and(|(t, _)| t == ticket) =>
            {
                child.frame.pending = Some((ticket, true));
                [0; 8]
            }
            [wire::INPUT_CAPACITY, epoch, channels, groups, 0, 0, 0, 0]
                if child.grant.valid()
                    && child.grant.epoch == epoch
                    && channels <= wire::INPUT_STREAMS as u64
                    && groups <= wire::INPUT_GROUPS as u64 =>
            {
                match child.limit_input(channels as usize, groups as usize) {
                    Ok(()) => [0, epoch, channels, groups, 0, 0, 0, 0],
                    Err(code) => wire::error(code),
                }
            }
            [wire::INPUT_DIRECTORY, epoch, revision, after, 0, 0, 0, 0] => {
                child.directory(epoch, revision, after)
            }
            [wire::INPUT_DESCRIPTION, epoch, revision, name, 0, 0, 0, 0] => {
                child.description(epoch, revision, name)
            }
            [wire::VIEW, 0, 0, 0, 0, 0, 0, 0] => child.view(),
            [wire::POLL, 0, 0, 0, 0, 0, 0, 0] if child.grant.valid() => child.poll(),
            [wire::POLL_TAGGED, id @ 1..=0xffff_ffff, 0, 0, 0, 0, 0, 0] => wire::tag_poll(
                if child.grant.valid() {
                    child.poll()
                } else {
                    wire::error(abi::DENIED)
                },
                id as u32,
            ),
            [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0] if child.grant.valid() => [
                0,
                child.surface_limit as u64,
                child.surfaces.iter().flatten().count() as u64,
                0,
                0,
                0,
                0,
                0,
            ],
            [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0]
                if child.grant.valid() && epoch == child.grant.epoch =>
            {
                child.next()
            }
            _ => wire::error(abi::DENIED),
        }
    }
}

impl<const N: usize> Default for World<N> {
    fn default() -> Self {
        Self::empty()
    }
}
