//! Bounded raw device bytes. Loss invalidates the stream before any later data.
pub struct ByteQueue {
    bytes: [u16; 64],
    head: usize,
    len: usize,
    lost: bool,
}
impl Default for ByteQueue {
    fn default() -> Self {
        Self::new()
    }
}
impl ByteQueue {
    pub const fn new() -> Self {
        Self {
            bytes: [0; 64],
            head: 0,
            len: 0,
            lost: false,
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn lose(&mut self) {
        self.len = 0;
        self.head = 0;
        self.lost = true;
    }
    pub fn push(&mut self, byte: u8) {
        self.push_tagged(u16::from(byte));
    }
    /// Preserve the controller source tag without interpreting device packets.
    pub fn push_tagged(&mut self, byte: u16) {
        if self.lost {
            return;
        }
        if self.len == self.bytes.len() {
            self.lose();
            return;
        }
        self.bytes[(self.head + self.len) % self.bytes.len()] = byte;
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<u64> {
        if self.lost {
            self.lost = false;
            return Some(cathedral_contracts::user::KEYBOARD_LOST);
        }
        if self.len == 0 {
            return None;
        }
        let byte = self.bytes[self.head];
        self.head = (self.head + 1) % self.bytes.len();
        self.len -= 1;
        Some(u64::from(byte))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interleaved_sources_remain_distinct_from_loss() {
        let mut queue = ByteQueue::new();
        queue.push(0xfa);
        queue.push_tagged(512 | 0xfa);
        queue.push(0);
        assert_eq!(queue.pop(), Some(0xfa));
        assert_eq!(queue.pop(), Some(512 | 0xfa));
        assert_eq!(queue.pop(), Some(0));
        queue.push_tagged(512 | 8);
        queue.lose();
        assert_eq!(queue.pop(), Some(256));
        assert_eq!(queue.pop(), None);
    }
    #[test]
    fn overflow_discards_partial_sequences_and_reset_discards_old_generation() {
        let mut queue = ByteQueue::new();
        for _ in 0..65 {
            queue.push(0xe0);
        }
        queue.push(0x4d);
        assert_eq!(queue.pop(), Some(256));
        assert_eq!(queue.pop(), None);
        for value in 0..100 {
            queue.push(value);
            assert_eq!(queue.pop(), Some(value as u64));
        }
        queue.push(42);
        queue.reset();
        assert_eq!(queue.pop(), None);
    }
}
