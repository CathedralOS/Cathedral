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
#[derive(Default)]
pub(super) struct Child {
    pub rect: Option<Rect>,
    pub keys: u64,
    pub grant: Grant,
    pub incarnation: u64,
    pub notify: bool,
    pub surfaces: [Option<wire::Packet>; 3],
    epoch: u64,
    queue: [wire::Packet; 8],
    len: usize,
}
impl Child {
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
        self.len = 0;
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
    pub fn push(&mut self, kind: u64, x: u64, y: u64, value: u64) -> bool {
        if !self.grant.valid() {
            return true;
        }
        if self.len == self.queue.len() {
            self.cancel();
            return false;
        }
        self.queue[self.len] = [0, self.grant.epoch, kind, x, y, value, 0, 0];
        self.len += 1;
        self.notify = true;
        true
    }
    pub fn cancel(&mut self) {
        self.len = 0;
        self.push(wire::CANCEL, 0, 0, 0);
    }
    pub fn next(&mut self) -> wire::Packet {
        if self.len == 0 {
            return [0, self.grant.epoch, wire::NONE, 0, 0, 0, 0, 0];
        }
        let event = self.queue[0];
        self.queue.copy_within(1..self.len, 0);
        self.len -= 1;
        event
    }
}
