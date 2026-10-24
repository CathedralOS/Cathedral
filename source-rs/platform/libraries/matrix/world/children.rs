//! Child-local extents, epochs and queues. Sibling changes cannot advance an epoch.
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
    group_names: [u16; wire::INPUT_GROUPS],
    next_group_name: u16,
    next_event: u64,
    group_last_event: [u64; wire::INPUT_GROUPS],
}
impl Child {
    pub const fn empty() -> Self {
        Self {
            visible: true,
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
            group_names: [0; wire::INPUT_GROUPS],
            next_group_name: 0,
            next_event: 0,
            group_last_event: [0; wire::INPUT_GROUPS],
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
        self.group_names = [0; wire::INPUT_GROUPS];
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
    pub fn push(&mut self, index: usize, channel: u32, group: usize, event: [u64; 4]) -> bool {
        if !self.grant.valid() {
            return true;
        }
        self.notify = true;
        self.next_event = self.next_event.checked_add(1).expect("input event order");
        let queue = &mut self.queues[index];
        if queue.source == 0 {
            // Child-local names reveal neither ancestor IDs nor sibling traffic.
            self.next_input_name += 1; // At most INPUT_STREAMS names per view binding.
            queue.channel = self.next_input_name;
            queue.source = channel;
            if self.group_names[group] == 0 {
                self.next_group_name += 1;
                self.group_names[group] = self.next_group_name;
            }
            queue.group = self.group_names[group];
        }
        debug_assert_eq!(queue.source, channel);
        let coalesce = queue.last_order() == Some(self.group_last_event[group]);
        self.group_last_event[group] = self.next_event;
        queue.push(event, self.next_event, coalesce)
    }
    pub fn cancel_stream(&mut self, index: usize, channel: u32) {
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
    pub fn cancel(&mut self) {
        for queue in &mut self.queues {
            self.next_event = self.next_event.checked_add(1).expect("input event order");
            queue.cancel(self.next_event);
        }
        self.group_last_event.fill(0);
        self.notify = true;
    }
    pub fn next(&mut self) -> wire::Packet {
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
                return self.queues[index].next(self.grant.epoch);
            }
        }
        [0, self.grant.epoch, wire::NONE, 0, 0, 0, 0, 0]
    }
    pub fn cancel_keys(&mut self, index: usize, channel: u32) {
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
        reply[5] = event[2]
            | (event[7] << 17)
            | (event[5] << 8)
            | (event[6] << 32)
            | if self.queues.iter().any(events::Events::pending) {
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
