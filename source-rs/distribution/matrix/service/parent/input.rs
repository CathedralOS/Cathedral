//! Negotiate inherited input storage before delegating local production. The
//! parent enforces the accepted ceiling; local free space alone is not a grant.
use super::*;
impl Parent {
    fn limit_input(&mut self, capacity: [usize; 2]) -> Result<(), Error> {
        assert!(self.upstream && !self.upstream_busy());
        let epoch = self.grant.epoch;
        let reply = self.call([
            wire::INPUT_CAPACITY,
            epoch,
            capacity[0] as u64,
            capacity[1] as u64,
            0,
            0,
            0,
            0,
        ])?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        if reply != [0, epoch, capacity[0] as u64, capacity[1] as u64, 0, 0, 0, 0] {
            return Err(Error(abi::IO_ERROR as i64));
        }
        self.input_capacity = capacity;
        Ok(())
    }
    pub(in crate::service) fn grant_input_producer(
        &mut self,
        world: &mut World,
        slot: usize,
        count: usize,
        rights: u64,
        minimum: [usize; 2],
    ) -> Result<(), Error> {
        if !world.input_ready() {
            return Err(Error(abi::BUSY as i64));
        }
        let capacity = world
            .inherited_input_headroom(count, 1)
            .map_err(|c| Error(c as i64))?;
        if capacity[0] < minimum[0] || capacity[1] < minimum[1] {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        self.limit_input(capacity)?;
        // A refusal here leaves the negotiated limit in place. The normal
        // restoration step widens it to the actual headroom on the next turn.
        world
            .grant_input_producer(slot, count, rights, capacity[0], capacity[1])
            .map_err(|c| Error(c as i64))
    }
    pub(in crate::service) fn restore_input_capacity(
        &mut self,
        world: &mut World,
    ) -> Result<(), Error> {
        if !self.upstream
            || !self.grant.valid()
            || self.interaction.is_none()
            || !world.input_ready()
        {
            return Ok(());
        }
        let available = world
            .inherited_input_headroom(0, 0)
            .map_err(|c| Error(c as i64))?;
        let capacity = [
            available[0].max(self.input_capacity[0]),
            available[1].max(self.input_capacity[1]),
        ];
        if capacity == self.input_capacity {
            return Ok(());
        }
        match self.limit_input(capacity) {
            Ok(()) => Ok(()),
            // View replacement may overtake a request. Refresh that view rather
            // than treating a refused capacity update as parent process death.
            Err(Error(code)) if code == abi::DENIED as i64 || code == abi::BUSY as i64 => {
                self.refresh_due = true;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}
