//! Bounded FIFO for one supplied input stream. Overflow cannot erase another stream.
use super::wire;

pub(super) struct Events {
    pub channel: u32,
    pub source: u32,
    queue: [[u64; 4]; 16],
    len: usize,
}
impl Events {
    pub const fn empty() -> Self {
        Self {
            channel: 0,
            source: 0,
            queue: [[0; 4]; 16],
            len: 0,
        }
    }
    pub fn pending(&self) -> bool {
        self.len != 0
    }
    pub fn push(&mut self, kind: u64, x: u64, y: u64, value: u64) -> bool {
        debug_assert!(self.channel != 0 && self.source != 0);
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
                return true;
            }
        }
        if self.len == self.queue.len() {
            self.cancel();
            return false;
        }
        self.queue[self.len] = [kind, x, y, value];
        self.len += 1;
        true
    }
    pub fn cancel(&mut self) {
        if self.channel != 0 {
            self.queue[0] = [wire::CANCEL, 0, 0, 0];
            self.len = 1;
        }
    }
    pub fn next(&mut self, epoch: u64) -> wire::Packet {
        debug_assert!(self.pending());
        let [kind, x, y, value] = self.queue[0];
        self.queue.copy_within(1..self.len, 0);
        self.len -= 1;
        [0, epoch, kind, x, y, value, self.channel as u64, 0]
    }
}
