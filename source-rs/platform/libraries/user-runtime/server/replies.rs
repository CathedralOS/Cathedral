//! Optional bounded copied-reply retention for two outstanding client requests.
//! One reply may occupy the kernel channel and one this caller-owned arena.
use super::*;

struct Pending {
    incarnation: u64,
    bytes: [u8; 64],
    len: usize,
}
pub struct Replies {
    pending: [Option<Pending>; MAX_CLIENTS],
    mask: u32,
}
impl Replies {
    pub const fn empty() -> Self {
        Self {
            pending: [const { None }; MAX_CLIENTS],
            mask: 0,
        }
    }
}
impl Server {
    /// Requires the clock grant. Retained replies retry at most once per tick
    /// when otherwise idle; writable-channel notification is not yet available.
    /// More than two unanswered requests may disconnect that client. Attachments
    /// and endpoint transfers retain their explicit caller-owned retry semantics.
    pub fn retain_replies(&mut self, arena: &'static mut Replies) {
        assert!(self.replies.is_none());
        self.replies = Some(arena);
    }

    pub(super) fn reply_paused(&self) -> u32 {
        self.replies.as_ref().map_or(0, |arena| arena.mask)
    }

    pub(super) fn retained_reply(&mut self, request: &Request, bytes: &[u8]) -> Result<(), Error> {
        let slot = request.client.unwrap();
        self.flush_reply(slot);
        if self.incarnation(slot) != Some(request.incarnation) {
            return Ok(());
        }
        if bytes.len() > 64 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        // Preserve FIFO. A full software slot means the client's bounded
        // outstanding-request contract has been exceeded, not a dropped success.
        if self.replies.as_ref().unwrap().pending[slot].is_some() {
            self.disconnect(slot);
            self.replies.as_mut().unwrap().pending[slot] = None;
            self.replies.as_mut().unwrap().mask &= !(1 << (slot + 1));
            return Ok(());
        }
        match request.reply.send(bytes) {
            Ok(()) => Ok(()),
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                let mut pending = Pending {
                    incarnation: request.incarnation,
                    bytes: [0; 64],
                    len: bytes.len(),
                };
                pending.bytes[..bytes.len()].copy_from_slice(bytes);
                self.replies.as_mut().unwrap().pending[slot] = Some(pending);
                self.replies.as_mut().unwrap().mask |= 1 << (slot + 1);
                Ok(())
            }
            Err(_) => {
                self.disconnect(slot);
                Ok(())
            }
        }
    }

    pub(super) fn flush_replies(&mut self) {
        if self.reply_paused() == 0 {
            return;
        }
        for slot in 0..self.count {
            self.flush_reply(slot);
        }
    }
    fn flush_reply(&mut self, slot: usize) {
        let Some(p) = self.replies.as_mut().and_then(|a| a.pending[slot].take()) else {
            return;
        };
        self.replies.as_mut().unwrap().mask &= !(1 << (slot + 1));
        let Some((receive, send)) = self.clients[slot] else {
            return;
        };
        if receive.raw() != p.incarnation {
            return;
        }
        match send.send(&p.bytes[..p.len]) {
            Ok(()) => (),
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                let arena = self.replies.as_mut().unwrap();
                arena.pending[slot] = Some(p);
                arena.mask |= 1 << (slot + 1);
            }
            Err(_) => self.disconnect(slot),
        }
    }
}
