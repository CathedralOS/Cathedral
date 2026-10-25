//! Supplied upstream binding: acquire once, then serialize parent snapshots.
//! This proof refreshes on local requests, not a frame/timer-driven desktop loop.
use super::*;
use cathedral_user_runtime::matrix::upstream;
impl Host {
    pub(super) fn initialize(&mut self, p: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        let grant = if self.parent.is_some() {
            self.refresh()?.ok_or(Error(abi::DENIED as i64))?
        } else {
            let grant = Grant {
                epoch: p[0],
                width: p[1],
                height: p[2],
                keys: p[3],
            };
            self.world.inherit(grant);
            grant
        };
        p[..4].copy_from_slice(&[grant.epoch, grant.width, grant.height, grant.keys]);
        Ok(())
    }
    pub(super) fn refresh(&mut self) -> Result<Option<Grant>, Error> {
        let Some(parent) = &mut self.parent else {
            return Err(Error(abi::DENIED as i64));
        };
        // Drain descendant retirement boundaries before accepting another channel.
        if !self.world.input_ready() {
            return Ok(None);
        }
        let connection = parent.interaction(1, 1)?;
        let reply = upstream::call(connection.pair, [wire::POLL, 0, 0, 0, 0, 0, 0, 0])?;
        // Preserve the retained local Request and all its packet/caller facts.
        Ok(Some(self.world.inherit_poll(reply)))
    }
}
