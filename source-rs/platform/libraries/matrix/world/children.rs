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
    // kind, x/key, y/state, buttons. Status/padding are not stored, and refresh
    // clears the queue before changing its epoch. Sixteen events use the same
    // 512 bytes as the former eight fully encoded reply packets.
    queue: [[u64; 4]; 16],
    len: usize,
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
            queue: [[0; 4]; 16],
            len: 0,
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
        // Keep the first report of a button state at its original coordinates.
        // Only replace a trailing motion report preceded by the same state;
        // never merge across a press, release, key, cancellation or epoch.
        if kind == wire::POINTER && self.len >= 2 {
            let tail = self.queue[self.len - 1];
            let before = self.queue[self.len - 2];
            if tail[0] == wire::POINTER
                && before[0] == wire::POINTER
                && tail[3] == value
                && before[3] == value
            {
                self.queue[self.len - 1][1] = x;
                self.queue[self.len - 1][2] = y;
                self.notify = true;
                return true;
            }
        }
        if self.len == self.queue.len() {
            self.cancel();
            return false;
        }
        self.queue[self.len] = [kind, x, y, value];
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
        let [kind, x, y, value] = self.queue[0];
        self.queue.copy_within(1..self.len, 0);
        self.len -= 1;
        [0, self.grant.epoch, kind, x, y, value, 0, 0]
    }
    pub fn poll(&mut self) -> wire::Packet {
        let event = self.next();
        let mut reply = self.view();
        reply[4] |= (self.surface_limit as u64) << wire::SURFACE_BITS;
        reply[5] = event[2] | (event[5] << 8) | if self.len != 0 { wire::MORE } else { 0 };
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
