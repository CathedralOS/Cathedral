// Admit upstream stream names, then route or cancel only that stream.
// Caller authentication belongs to the service; producer.rs checks explicitly
// granted child connections before translating their private channel ordinals.
use super::*;

pub(super) struct Input {
    streams: [Stream; wire::INPUT_STREAMS],
    groups: [Group; wire::INPUT_GROUPS],
    last_local_stream: u32,
    last_local_group: u32,
}
impl Input {
    pub const fn empty() -> Self {
        Self {
            streams: [Stream::empty(); wire::INPUT_STREAMS],
            groups: [Group::empty(); wire::INPUT_GROUPS],
            last_local_stream: 0,
            last_local_group: 0,
        }
    }
    fn admit(&mut self, id: Name) -> Result<usize, u64> {
        if !id.valid() {
            return Err(abi::INVALID_ARGUMENT);
        }
        if let Some(slot) = self.streams.iter().position(|s| s.id == id) {
            return if self.streams[slot].retiring {
                Err(abi::PEER_CLOSED)
            } else {
                Ok(slot)
            };
        }
        self.bind(id, id)
    }
    pub(super) fn bind(&mut self, id: Name, group: Name) -> Result<usize, u64> {
        if !id.valid() || !group.valid() {
            return Err(abi::INVALID_ARGUMENT);
        }
        if let Some(slot) = self.streams.iter().position(|s| s.id == id) {
            return if self.streams[slot].retiring {
                Err(abi::PEER_CLOSED)
            } else if self.groups[self.streams[slot].group].id == group {
                Ok(slot)
            } else {
                Err(abi::BUSY)
            };
        }
        // Local admission allocates monotonically, so a stale producer name
        // cannot recreate a retired channel without an unbounded tombstone set.
        if !id.inherited && id.value <= self.last_local_stream {
            return Err(abi::PEER_CLOSED);
        }
        let slot = self
            .streams
            .iter()
            .position(|s| !s.id.valid())
            .ok_or(abi::NO_MEMORY)?;
        let group_slot = self
            .groups
            .iter()
            .position(|g| g.id == group)
            .or_else(|| self.groups.iter().position(|g| !g.id.valid()))
            .ok_or(abi::NO_MEMORY)?;
        if self.groups[group_slot].retiring {
            return Err(abi::PEER_CLOSED);
        }
        self.groups[group_slot].id = group;
        // Only an explicit upstream group retirement ends an inherited route.
        // Losing its last channel does not erase the remembered local selection.
        self.groups[group_slot].retained |= group.inherited;
        if !id.inherited {
            self.last_local_stream = id.value;
        }
        if !group.inherited {
            self.last_local_group = self.last_local_group.max(group.value);
        }
        self.streams[slot] = Stream {
            id,
            group: group_slot,
            ..Stream::empty()
        };
        Ok(slot)
    }
    pub fn forget(&mut self, child: usize) {
        for (index, group) in self.groups.iter_mut().enumerate() {
            if group.focus == Some(child) {
                group.focus = None;
                for stream in &mut self.streams {
                    if stream.group == index {
                        stream.suppressed |= stream.held;
                    }
                }
            }
        }
        for stream in &mut self.streams {
            stream.forget(child);
        }
    }
}

impl<const N: usize> World<N> {
    /// Associate a newly supplied stream before forwarding its first event.
    /// IDs are names within authenticated authority, not capabilities by themselves.
    /// Trusted local adapters supply monotonically increasing fresh names;
    /// repeating a live binding is harmless. Retired names cannot be recreated.
    /// Dynamic providers should use registration to allocate those names.
    /// Reassociation of a live channel is not implemented.
    pub fn bind_input_stream(&mut self, id: u32, group: u32) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        self.reap_input();
        self.input
            .bind(Name::local(id), Name::local(group))
            .map(|_| ())
    }
    /// Entire upstream binding is interrupted; preserve local selections.
    pub fn cancel(&mut self) {
        for index in 0..wire::INPUT_STREAMS {
            if self.input.streams[index].id.valid() && !self.input.streams[index].retiring {
                self.cancel_stream_index(index);
            }
        }
    }
    fn cancel_stream_index(&mut self, index: usize) {
        let id = self.input.streams[index].id;
        for child in &mut self.children {
            child.cancel_stream(index, id);
        }
        self.input.streams[index].cancel();
    }
    /// Authenticated upstream lifecycle event, not a child-selected stream ID.
    pub fn cancel_stream(&mut self, id: u32) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        let index = self.input.admit(Name::local(id))?;
        self.cancel_stream_index(index);
        Ok(())
    }
    /// A delivered CANCEL already terminates the upstream stream's held state.
    /// Its sender suppresses residual hardware holds until release. Waiting for
    /// those releases here would suppress the first fresh press after recovery.
    pub(super) fn inherit_stream_cancel(&mut self, id: Name) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        let index = self.input.admit(id)?;
        self.cancel_stream_index(index);
        let group = self.input.streams[index].group;
        let description = self.input.streams[index].description;
        let description_locked = self.input.streams[index].description_locked;
        self.input.streams[index] = Stream {
            id,
            group,
            description,
            description_locked,
            ..Stream::empty()
        };
        Ok(())
    }
    pub(super) fn inherit_key_cancel(&mut self, id: Name) -> Result<(), u64> {
        let index = self.input.admit(id)?;
        for child in &mut self.children {
            child.cancel_keys(index, id);
        }
        self.input.streams[index].held = 0;
        self.input.streams[index].suppressed = 0;
        Ok(())
    }
    /// Existing single-source adapter; this does not grant a child injection rights.
    pub fn pointer(&mut self, x: u64, y: u64, buttons: u64) {
        let _ = self.pointer_stream(wire::SINGLE_INPUT_STREAM, x, y, buttons);
    }
    pub fn key(&mut self, key: u64, state: u64) {
        let _ = self.key_stream(wire::SINGLE_INPUT_STREAM, key, state);
    }
}
