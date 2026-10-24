//! Observe only this receiver's admitted aliases. Iteration cannot mint names,
//! consume input or reveal the routing owner's source IDs or sibling bindings.
use super::*;
impl Child {
    pub(super) fn directory_changed(&mut self) {
        if self.directory_revision != 0 {
            self.directory_revision = self.directory_revision.checked_add(1).unwrap_or(0);
        }
        self.notify = true;
    }
    pub fn directory(&self, epoch: u64, revision: u64, after: u64) -> wire::Packet {
        if !self.grant.valid()
            || epoch != self.grant.epoch
            || after > u32::MAX as u64
            || (revision == 0 && after != 0)
        {
            return wire::error(abi::DENIED);
        }
        if self.directory_revision == 0 {
            return wire::error(abi::NO_MEMORY);
        }
        if revision != 0 && revision != self.directory_revision {
            return wire::error(abi::BUSY);
        }
        let next = self
            .queues
            .iter()
            .filter(|q| q.channel as u64 > after)
            .min_by_key(|q| q.channel);
        match next {
            Some(q) => [
                0,
                epoch,
                self.directory_revision,
                q.channel as u64,
                q.group as u64,
                if q.retiring() {
                    wire::CHANNEL_RETIRING
                } else {
                    wire::CHANNEL_LIVE
                },
                self.grant.keys,
                0,
            ],
            None => [0, epoch, self.directory_revision, 0, 0, 0, 0, 0],
        }
    }
}
