//! Provider-side admission and retirement. The service authenticates callers
//! and grants before entering here; numeric names are never caller authority.
use super::*;

impl<const N: usize> World<N> {
    pub(in crate::world) fn input_owner_command(&mut self, words: wire::Packet) -> wire::Packet {
        let result = match words {
            [wire::INPUT_GROUP_CREATE, 0, 0, 0, 0, 0, 0, 0] => self.register_input_group(),
            [wire::INPUT_CHANNEL_CREATE, group, 0, 0, 0, 0, 0, 0] if group <= u32::MAX as u64 => {
                self.register_input_stream(group as u32)
            }
            [wire::INPUT_GROUP_RELEASE, group, 0, 0, 0, 0, 0, 0] if group <= u32::MAX as u64 => {
                self.release_input_group(group as u32).map(|()| 0)
            }
            [wire::INPUT_CHANNEL_RETIRE, channel, 0, 0, 0, 0, 0, 0]
                if channel <= u32::MAX as u64 =>
            {
                self.retire_input_stream(channel as u32).map(|()| 0)
            }
            [wire::INPUT_SELECT, group, child, 0, 0, 0, 0, 0]
                if group <= u32::MAX as u64 && child <= N as u64 =>
            {
                self.select_input_group(group as u32, child.checked_sub(1).map(|s| s as usize))
                    .map(|()| 0)
            }
            [wire::INPUT_POINTER, channel, x, y, buttons, 0, 0, 0]
                if self.registered_channel(channel) =>
            {
                self.pointer_stream(channel as u32, x, y, buttons)
                    .map(|()| 0)
            }
            [wire::INPUT_KEY, channel, key, state, 0, 0, 0, 0]
                if self.registered_channel(channel) =>
            {
                self.key_stream(channel as u32, key, state).map(|()| 0)
            }
            _ => Err(abi::DENIED),
        };
        match result {
            Ok(name) => [0, name as u64, 0, 0, 0, 0, 0, 0],
            Err(code) => wire::error(code),
        }
    }
    fn registered_channel(&self, id: u64) -> bool {
        id != 0
            && id <= u32::MAX as u64
            && self.input.streams.iter().any(|s| {
                s.id == Name::local(id as u32) && !s.retiring && self.input.groups[s.group].retained
            })
    }
    pub fn register_input_group(&mut self) -> Result<u32, u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        self.reap_input();
        let id = self
            .input
            .last_local_group
            .checked_add(1)
            .ok_or(abi::NO_MEMORY)?;
        let group = self
            .input
            .groups
            .iter_mut()
            .find(|g| !g.id.valid())
            .ok_or(abi::NO_MEMORY)?;
        *group = Group {
            id: Name::local(id),
            focus: None,
            retained: true,
            retiring: false,
        };
        self.input.last_local_group = id;
        Ok(id)
    }
    pub fn register_input_stream(&mut self, group: u32) -> Result<u32, u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        self.reap_input();
        if !self
            .input
            .groups
            .iter()
            .any(|g| g.id == Name::local(group) && g.retained && !g.retiring)
        {
            return Err(abi::DENIED);
        }
        let id = self
            .input
            .last_local_stream
            .checked_add(1)
            .ok_or(abi::NO_MEMORY)?;
        self.input.bind(Name::local(id), Name::local(group))?;
        Ok(id)
    }
    pub fn release_input_group(&mut self, id: u32) -> Result<(), u64> {
        self.retire_group_named(Name::local(id))
    }
    pub(in crate::world) fn retire_group_named(&mut self, id: Name) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        self.reap_input();
        let index = self
            .input
            .groups
            .iter()
            .position(|g| g.id == id && g.retained)
            .ok_or(abi::DENIED)?;
        if self
            .input
            .streams
            .iter()
            .any(|s| s.id.valid() && s.group == index)
        {
            return Err(abi::BUSY);
        }
        self.input.groups[index].retiring = true;
        for child in &mut self.children {
            child.retire_input_group(index, id);
        }
        self.reap_input();
        Ok(())
    }
    pub fn retire_input_stream(&mut self, id: u32) -> Result<(), u64> {
        self.retire_named(Name::local(id))
    }
    pub(in crate::world) fn retire_named(&mut self, id: Name) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        let index = self
            .input
            .streams
            .iter()
            .position(|s| s.id == id && id.valid())
            .ok_or(abi::DENIED)?;
        let stream = &mut self.input.streams[index];
        if stream.retiring {
            return Ok(());
        }
        stream.retiring = true;
        stream.cancel();
        for child in &mut self.children {
            child.retire_stream(index, id);
        }
        self.reap_input();
        Ok(())
    }
    pub(in crate::world) fn reap_input(&mut self) {
        for (index, stream) in self.input.streams.iter_mut().enumerate() {
            if stream.retiring
                && self
                    .children
                    .iter()
                    .all(|c| !c.retains_input(index, stream.id))
            {
                *stream = Stream::empty();
            }
        }
        for (index, group) in self.input.groups.iter_mut().enumerate() {
            if group.id.valid()
                && !group.retained
                && !group.retiring
                && !self
                    .input
                    .streams
                    .iter()
                    .any(|s| s.id.valid() && s.group == index)
            {
                group.retiring = true;
                for child in &mut self.children {
                    child.retire_input_group(index, group.id);
                }
            }
            if group.retiring
                && self
                    .children
                    .iter()
                    .all(|c| !c.retains_input_group(index, group.id))
            {
                *group = Group::empty();
            }
        }
    }
    /// Before taking another upstream input snapshot, let descendants drain
    /// retirement boundaries. Local requests/other channels remain serviceable.
    /// This bounded backpressure prevents a new parent channel from arriving
    /// while the same slot is still retained by an old child-visible binding.
    pub fn input_ready(&mut self) -> bool {
        self.reap_input();
        !self.input.streams.iter().any(|s| s.retiring)
            && !self.input.groups.iter().any(|g| g.retiring)
    }
}
