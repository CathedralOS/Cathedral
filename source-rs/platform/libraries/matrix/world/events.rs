//! Bounded FIFO for one supplied input stream. Overflow cannot erase another stream.
use super::wire;

pub(super) struct Events {
    pub channel: u32,
    pub source: u32,
    pub group: u16,
    // Sixteen ordinary events plus a reserved keyboard-cancellation boundary.
    queue: [[u64; 5]; 17],
    len: usize,
}
impl Events {
    pub const fn empty() -> Self {
        Self {
            channel: 0,
            source: 0,
            group: 0,
            queue: [[0; 5]; 17],
            len: 0,
        }
    }
    pub fn pending(&self) -> bool {
        self.len != 0
    }
    pub fn first_order(&self) -> Option<u64> {
        self.pending().then_some(self.queue[0][4])
    }
    pub fn last_order(&self) -> Option<u64> {
        self.len.checked_sub(1).map(|i| self.queue[i][4])
    }
    pub fn push(&mut self, [kind, x, y, value]: [u64; 4], order: u64, coalesce: bool) -> bool {
        debug_assert!(self.channel != 0 && self.source != 0);
        if coalesce && kind == wire::POINTER && self.len >= 2 {
            let tail = self.queue[self.len - 1];
            let before = self.queue[self.len - 2];
            if tail[0] == wire::POINTER
                && before[0] == wire::POINTER
                && tail[3] == value
                && before[3] == value
            {
                self.queue[self.len - 1][1] = x;
                self.queue[self.len - 1][2] = y;
                self.queue[self.len - 1][4] = order;
                return true;
            }
        }
        if self.len >= 16 {
            self.cancel(order);
            return false;
        }
        self.queue[self.len] = [kind, x, y, value, order];
        self.len += 1;
        true
    }
    pub fn cancel(&mut self, order: u64) {
        if self.channel != 0 {
            self.queue[0] = [wire::CANCEL, 0, 0, 0, order];
            self.len = 1;
        }
    }
    pub fn next(&mut self, epoch: u64) -> wire::Packet {
        debug_assert!(self.pending());
        let [kind, x, y, value, _order] = self.queue[0];
        self.queue.copy_within(1..self.len, 0);
        self.len -= 1;
        [
            0,
            epoch,
            kind,
            x,
            y,
            value,
            self.channel as u64,
            self.group as u64,
        ]
    }
    pub fn cancel_keys(&mut self, order: u64) {
        if self.channel == 0 {
            return;
        }
        let mut keep = 0;
        for index in 0..self.len {
            if !matches!(self.queue[index][0], wire::KEY | wire::CANCEL_KEYS) {
                self.queue[keep] = self.queue[index];
                keep += 1;
            }
        }
        self.queue[keep] = [wire::CANCEL_KEYS, 0, 0, 0, order];
        self.len = keep + 1;
    }
}
