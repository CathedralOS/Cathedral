// Child-local extents, epochs and queues. Sibling changes cannot advance an epoch.
// Directory observation is in directory.rs; input delivery changes its revision.
use super::input::Name;
use super::*;
#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub x: u64,
    pub y: u64,
    pub width: u64,
    pub height: u64,
}
impl Rect {
    pub(super) fn valid(self) -> bool {
        self.width > 0
            && self.height > 0
            && self.x < wire::MAX_EXTENT
            && self.y < wire::MAX_EXTENT
            && self.width <= wire::MAX_EXTENT
            && self.height <= wire::MAX_EXTENT
    }
}
pub(super) struct Child {
    pub visible: bool,
    frames: bool,
    pub rect: Option<Rect>,
    pub keys: u64,
    pub grant: Grant,
    pub incarnation: u64,
    pub notify: bool,
    pub surfaces: [Option<surface::Surface>; wire::MAX_SURFACES],
    pub surface_base: usize,
    pub surface_limit: usize,
    pub next_surface: u64,
    epoch: u64,
    queues: [events::Events; wire::INPUT_STREAMS],
    next_stream: usize,
    next_input_name: u32,
    directory_revision: u64,
    group_names: [u16; wire::INPUT_GROUPS],
    group_sources: [Name; wire::INPUT_GROUPS],
    group_retired: [bool; wire::INPUT_GROUPS],
    next_group_name: u16,
    next_event: u64,
    group_last_event: [u64; wire::INPUT_GROUPS],
    pub input_capacity: [usize; 2],
}
impl Child {
    pub const fn empty() -> Self {
        Self {
            visible: true,
            frames: true,
            rect: None,
            keys: 0,
            grant: Grant {
                epoch: 0,
                width: 0,
                height: 0,
                keys: 0,
            },
            incarnation: 0,
            notify: false,
            surfaces: [None; wire::MAX_SURFACES],
            surface_base: 0,
            surface_limit: 0,
            next_surface: 0,
            epoch: 0,
            queues: [const { events::Events::empty() }; wire::INPUT_STREAMS],
            next_stream: 0,
            next_input_name: 0,
            directory_revision: 1,
            group_names: [0; wire::INPUT_GROUPS],
            group_sources: [Name::EMPTY; wire::INPUT_GROUPS],
            group_retired: [false; wire::INPUT_GROUPS],
            next_group_name: 0,
            next_event: 0,
            group_last_event: [0; wire::INPUT_GROUPS],
            input_capacity: [wire::INPUT_STREAMS, wire::INPUT_GROUPS],
        }
    }

    pub fn refresh(&mut self, parent: Grant, force: bool) {
        let (width, height, keys) = match self.rect {
            Some(r) if parent.valid() => (
                r.width.min(parent.width.saturating_sub(r.x)),
                r.height.min(parent.height.saturating_sub(r.y)),
                self.keys & parent.keys,
            ),
            _ => (0, 0, 0),
        };
        if !force && (width, height, keys) == (self.grant.width, self.grant.height, self.grant.keys)
        {
            return;
        }
        self.queues = [const { events::Events::empty() }; wire::INPUT_STREAMS];
        self.next_stream = 0;
        self.next_input_name = 0;
        self.directory_revision = 1;
        self.group_names = [0; wire::INPUT_GROUPS];
        self.group_sources = [Name::EMPTY; wire::INPUT_GROUPS];
        self.group_retired = [false; wire::INPUT_GROUPS];
        self.next_group_name = 0;
        self.next_event = 0;
        self.group_last_event = [0; wire::INPUT_GROUPS];
        self.notify = true;
        let Some(epoch) = self.epoch.checked_add(1) else {
            self.grant = Grant::default();
            return;
        };
        self.epoch = epoch;
        self.grant = Grant {
            epoch,
            width,
            height,
            keys,
        };
    }
    pub fn view(&self) -> wire::Packet {
        let g = self.grant;
        if !g.valid() {
            return wire::error(abi::DENIED);
        }
        [0, g.epoch, g.width, g.height, g.keys, 0, 0, 0]
    }
    pub fn demand(&mut self, frames: bool) {
        if self.frames != frames {
            self.frames = frames;
            self.notify = true;
        }
    }
    pub fn push(
        &mut self,
        index: usize,
        channel: Name,
        group: usize,
        group_name: Name,
        description: cathedral_platform_contracts::input::description::Description,
        event: [u64; 4],
    ) -> Result<bool, u64> {
        if !self.grant.valid() {
            return Ok(true);
        }
        self.can_push(index, group, group_name)?;
        self.notify = true;
        self.next_event = self.next_event.checked_add(1).expect("input event order");
        let queue = &mut self.queues[index];
        let created = !queue.source.valid();
        if created {
            // Child-local names reveal neither ancestor IDs nor sibling traffic.
            let next_channel = self.next_input_name.checked_add(1).ok_or(abi::NO_MEMORY)?;
            if self.group_sources[group] != group_name {
                self.next_group_name = self
                    .next_group_name
                    .checked_add(1)
                    .filter(|name| *name <= 0x7fff)
                    .ok_or(abi::NO_MEMORY)?;
                self.group_names[group] = self.next_group_name;
                self.group_sources[group] = group_name;
            }
            self.next_input_name = next_channel;
            queue.channel = next_channel;
            queue.source = channel;
            queue.group = self.group_names[group];
            queue.description = description;
            queue.description_pending = description
                != cathedral_platform_contracts::input::description::Description::UNKNOWN;
        }
        debug_assert_eq!(queue.source, channel);
        let coalesce = queue.last_order() == Some(self.group_last_event[group]);
        self.group_last_event[group] = self.next_event;
        let delivered = queue.push(event, self.next_event, coalesce);
        if created {
            self.directory_changed();
        }
        Ok(delivered)
    }
    /// Preflight before routing changes focus, capture or held state. Retained
    /// names still consume capacity even when their event queue is empty.
    pub fn can_push(&self, index: usize, group: usize, name: Name) -> Result<(), u64> {
        if self.queues[index].source.valid() {
            return Ok(());
        }
        if self.queues.iter().filter(|q| q.source.valid()).count() >= self.input_capacity[0]
            || self.next_input_name == u32::MAX
        {
            return Err(abi::NO_MEMORY);
        }
        if self.group_sources[group] != name
            && (self.group_sources.iter().filter(|g| g.valid()).count() >= self.input_capacity[1]
                || self.next_group_name == 0x7fff)
        {
            return Err(abi::NO_MEMORY);
        }
        Ok(())
    }
    pub fn limit_input(&mut self, channels: usize, groups: usize) -> Result<(), u64> {
        if channels > wire::INPUT_STREAMS || groups > wire::INPUT_GROUPS {
            return Err(abi::DENIED);
        }
        if self.queues.iter().filter(|q| q.source.valid()).count() > channels
            || self.group_sources.iter().filter(|g| g.valid()).count() > groups
        {
            return Err(abi::BUSY);
        }
        self.input_capacity = [channels, groups];
        Ok(())
    }
    pub fn cancel_stream(&mut self, index: usize, channel: Name) {
        let queue = &mut self.queues[index];
        if queue.source == channel {
            self.next_event = self.next_event.checked_add(1).expect("input event order");
            queue.cancel(self.next_event);
            // Cancellation is an ordering barrier even if another channel is
            // the next one to publish to this group.
            self.group_last_event.fill(0);
            self.notify = true;
        }
    }
    pub fn retains_input(&self, index: usize, channel: Name) -> bool {
        self.queues[index].source == channel
    }
    pub fn retire_stream(&mut self, index: usize, channel: Name) {
        if self.queues[index].source == channel {
            self.next_event = self.next_event.checked_add(1).expect("input event order");
            let changed = !self.queues[index].retiring();
            self.queues[index].retire(self.next_event);
            if changed {
                self.directory_changed();
            }
            self.group_last_event.fill(0);
            self.notify = true;
        }
    }
    pub fn retire_input_group(&mut self, index: usize, group: Name) {
        if self.group_sources[index] == group {
            self.group_retired[index] = true;
            self.notify = true;
        }
    }
    pub fn retains_input_group(&self, index: usize, group: Name) -> bool {
        self.group_sources[index] == group && self.group_retired[index]
    }
    pub fn cancel(&mut self) {
        for queue in &mut self.queues {
            self.next_event = self.next_event.checked_add(1).expect("input event order");
            queue.cancel(self.next_event);
        }
        self.group_last_event.fill(0);
        self.notify = true;
    }
    pub fn next(&mut self) -> wire::Packet {
        // Group retirement is admitted only after all its channel queues drain.
        // Up to INPUT_GROUPS control boundaries precede independent group input.
        if let Some(index) = self.group_retired.iter().position(|ended| *ended) {
            let group = self.group_names[index];
            self.group_retired[index] = false;
            self.group_names[index] = 0;
            self.group_sources[index] = Name::EMPTY;
            self.group_last_event[index] = 0;
            return [
                0,
                self.grant.epoch,
                wire::GROUP_RETIRED,
                0,
                0,
                0,
                0,
                group as u64,
            ];
        }
        // Only a group's earliest event is eligible. Round-robin chooses among
        // eligible streams, so independent groups progress without reordering
        // a selecting pointer and a subsequent key in the same group.
        for offset in 0..wire::INPUT_STREAMS {
            let index = (self.next_stream + offset) % wire::INPUT_STREAMS;
            let candidate = &self.queues[index];
            if let Some(order) = candidate.first_order()
                && !self.queues.iter().any(|q| {
                    q.group == candidate.group
                        && q.first_order().is_some_and(|earlier| earlier < order)
                })
            {
                self.next_stream = (index + 1) % wire::INPUT_STREAMS;
                let event = self.queues[index].next(self.grant.epoch);
                if event[2] == wire::RETIRED {
                    self.directory_changed();
                }
                return event;
            }
        }
        [0, self.grant.epoch, wire::NONE, 0, 0, 0, 0, 0]
    }
    pub fn cancel_keys(&mut self, index: usize, channel: Name) {
        let queue = &mut self.queues[index];
        if queue.source == channel {
            self.next_event = self.next_event.checked_add(1).expect("input event order");
            queue.cancel_keys(self.next_event);
            self.group_last_event.fill(0);
            self.notify = true;
        }
    }
    pub fn poll(&mut self) -> wire::Packet {
        let event = self.next();
        let mut reply = self.view();
        reply[4] |= (self.surface_limit as u64) << wire::SURFACE_BITS;
        if !self.frames {
            reply[4] |= wire::NO_FRAMES;
        }
        reply[5] = event[2]
            | (event[7] << 17)
            | (event[5] << 8)
            | (event[6] << 32)
            | if self.queues.iter().any(events::Events::pending)
                || self.group_retired.iter().any(|ended| *ended)
            {
                wire::MORE
            } else {
                0
            };
        reply[6] = event[3];
        reply[7] = event[4];
        reply
    }
}

impl Default for Child {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_compact_names_refuse_instead_of_wrapping_or_publishing() {
        let mut child = Child::empty();
        child.rect = Some(Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 20,
        });
        child.refresh(
            Grant {
                epoch: 1,
                width: 20,
                height: 20,
                keys: 0,
            },
            false,
        );
        for (channel, group) in [(u32::MAX, 0), (0, 0x7fff)] {
            child.next_input_name = channel;
            child.next_group_name = group;
            assert_eq!(
                child.push(
                    0,
                    Name::local(1),
                    0,
                    Name::local(1),
                    cathedral_platform_contracts::input::description::Description::UNKNOWN,
                    [wire::POINTER, 1, 1, 1]
                ),
                Err(abi::NO_MEMORY)
            );
            assert!(!child.queues[0].source.valid());
            assert_eq!(child.next()[2], wire::NONE);
        }
    }
}
