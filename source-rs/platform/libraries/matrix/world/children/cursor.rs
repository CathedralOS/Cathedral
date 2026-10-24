//! Accept a preference only for a pointer alias actually delivered to this view.
use super::*;
impl Child {
    pub fn cursor_preference(&mut self, epoch: u64, channel: u64, mode: u64) -> wire::Packet {
        if !self.grant.valid()
            || epoch != self.grant.epoch
            || channel == 0
            || mode > wire::CURSOR_HIDDEN
        {
            return wire::error(abi::DENIED);
        }
        let Some(index) = self
            .queues
            .iter()
            .position(|q| q.channel as u64 == channel && !q.retiring())
        else {
            return wire::error(abi::DENIED);
        };
        let Some(preference) = &mut self.cursor[index] else {
            return wire::error(abi::DENIED);
        };
        *preference = mode == wire::CURSOR_HIDDEN;
        [0; 8]
    }
}
