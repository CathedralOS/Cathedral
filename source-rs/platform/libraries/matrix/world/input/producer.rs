//! Reserve a bounded producer grant, authenticate its connection and generation,
//! then translate only its own channel ordinals into routing operations.
use super::*;

#[derive(Clone, Copy)]
pub(in crate::world) struct Producer {
    incarnation: u64,
    epoch: u64,
    generation: u64,
    channels: [u32; wire::INPUT_STREAMS],
    count: usize,
    group: u32,
    rights: u64,
}
impl Producer {
    pub const fn empty() -> Self {
        Self {
            incarnation: 0,
            epoch: 0,
            generation: 0,
            channels: [0; wire::INPUT_STREAMS],
            count: 0,
            group: 0,
            rights: 0,
        }
    }
}

impl<const N: usize> World<N> {
    /// Owner policy supplies capacity to leave available for inherited input.
    /// Already admitted inherited names count toward that reservation. Trusted
    /// fixed local adapters must be bound before granting dynamic producers.
    pub fn grant_input_producer(
        &mut self,
        slot: usize,
        count: usize,
        rights: u64,
        inherited_channels: usize,
        inherited_groups: usize,
    ) -> Result<(), u64> {
        self.reap_input();
        let child = self.children.get(slot).ok_or(abi::DENIED)?;
        if !self.parent.valid()
            || !child.grant.valid()
            || child.incarnation == 0
            || count == 0
            || count > wire::INPUT_STREAMS
            || rights & !wire::PRODUCER_FOCUS != 0
            || inherited_channels > wire::INPUT_STREAMS
            || inherited_groups > wire::INPUT_GROUPS
        {
            return Err(abi::DENIED);
        }
        if self.producers[slot].generation != 0 {
            return Err(abi::BUSY);
        }
        let streams = &self.input.streams;
        let groups = &self.input.groups;
        let remaining_channels = inherited_channels.saturating_sub(
            streams
                .iter()
                .filter(|s| s.id.valid() && s.id.inherited)
                .count(),
        );
        let remaining_groups = inherited_groups.saturating_sub(
            groups
                .iter()
                .filter(|g| g.id.valid() && g.id.inherited)
                .count(),
        );
        if streams.iter().filter(|s| !s.id.valid()).count() < count + remaining_channels
            || groups.iter().filter(|g| !g.id.valid()).count() < 1 + remaining_groups
            || self
                .input
                .last_local_stream
                .checked_add(count as u32)
                .is_none()
            || self.input.last_local_group == u32::MAX
        {
            return Err(abi::NO_MEMORY);
        }
        let generation = self.next_producer[slot]
            .checked_add(1)
            .ok_or(abi::NO_MEMORY)?;
        let mut producer = Producer {
            incarnation: child.incarnation,
            epoch: child.grant.epoch,
            generation,
            count,
            rights,
            ..Producer::empty()
        };
        producer.group = self.register_input_group()?;
        for channel in &mut producer.channels[..count] {
            // Capacity and both identity counters were preflighted above. No
            // caller or concurrent actor can mutate this single-owner world.
            *channel = self
                .register_input_stream(producer.group)
                .expect("preflighted producer channel reservation");
        }
        self.next_producer[slot] = generation;
        self.producers[slot] = producer;
        Ok(())
    }

    /// Stop accepting immediately; retirement storage remains charged until
    /// recipients consume the boundary. Regrant cannot bypass that capacity.
    pub fn revoke_input_producer(&mut self, slot: usize) {
        let Some(producer) = self.producers.get_mut(slot) else {
            return;
        };
        let producer = core::mem::replace(producer, Producer::empty());
        if producer.generation == 0 {
            return;
        }
        for channel in &producer.channels[..producer.count] {
            let id = Name::local(*channel);
            if let Some(index) = self.input.streams.iter().position(|s| s.id == id) {
                let stream = &mut self.input.streams[index];
                stream.retiring = true;
                stream.cancel();
                for child in &mut self.children {
                    child.retire_stream(index, id);
                }
            }
        }
        if let Some(group) = self
            .input
            .groups
            .iter_mut()
            .find(|g| g.id == Name::local(producer.group))
        {
            // The ordinary reaper emits GROUP_RETIRED only after every channel
            // boundary drains. No caller has to retry group release after death.
            group.retained = false;
        }
    }

    pub(super) fn reap_producers(&mut self) {
        for slot in 0..N {
            let producer = self.producers[slot];
            if producer.generation != 0
                && !self.binding_matches(slot, producer.incarnation, producer.epoch)
            {
                self.revoke_input_producer(slot);
            }
        }
    }

    pub(in crate::world) fn producer_request(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
    ) -> wire::Packet {
        self.reap_input();
        let producer = self.producers[slot];
        if producer.generation == 0 || producer.incarnation != incarnation {
            return wire::error(abi::DENIED);
        }
        if words == [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0] {
            return [
                0,
                producer.generation,
                self.parent.width,
                self.parent.height,
                self.parent.keys,
                producer.count as u64,
                producer.rights,
                0,
            ];
        }
        if words[1] != producer.generation {
            return wire::error(abi::DENIED);
        }
        if words
            == [
                wire::PRODUCER_RELEASE,
                producer.generation,
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        {
            self.revoke_input_producer(slot);
            return [0; 8];
        }
        let Some(index) = words[2]
            .checked_sub(1)
            .filter(|i| *i < producer.count as u64)
        else {
            return wire::error(abi::DENIED);
        };
        let name = Name::local(producer.channels[index as usize]);
        let result = match words {
            [wire::PRODUCER_DESCRIBE, _, _, a, b, controls, 0, 0] => {
                match cathedral_platform_contracts::input::description::Description::from_words([
                    a, b, controls,
                ]) {
                    Some(description) => self.describe_input(name, description),
                    None => Err(abi::INVALID_ARGUMENT),
                }
            }
            [wire::PRODUCER_POINTER, _, _, x, y, buttons, 0, 0]
                if x < self.parent.width && y < self.parent.height =>
            {
                self.pointer_routed(
                    name,
                    x,
                    y,
                    buttons,
                    producer.rights & wire::PRODUCER_FOCUS != 0,
                )
            }
            [wire::PRODUCER_KEY, _, _, key, state, 0, 0, 0] => self.key_named(name, key, state),
            _ => Err(abi::DENIED),
        };
        wire::error(result.err().unwrap_or(0))
    }
}
