//! One identified upstream snapshot, progressed by the service loop. Lateness
//! keeps the request alive; unrelated replies cannot alter grants or input.
use super::*;
pub(super) struct Pending {
    id: u32,
    deadline: u64,
    late: bool,
}
impl Parent {
    #[cfg(feature = "interaction-delay")]
    pub(in crate::service) fn refresh_late(&self) -> bool {
        self.polling.as_ref().is_some_and(|p| p.late)
    }
    pub(in crate::service) fn refreshing(&self) -> bool {
        self.polling.is_some()
    }
    pub(in crate::service) fn refresh_wait(&self) -> Option<(Handle, Option<u64>)> {
        self.polling
            .as_ref()
            .map(|p| (self.pair.unwrap().1, (!p.late).then_some(p.deadline)))
    }
    pub fn refresh_notified(&mut self, world: &mut World) -> Result<(), Error> {
        if !self.upstream || self.wake.is_none() || self.pair.is_none() || !world.input_ready() {
            return Ok(());
        }
        assert!(!self.upstream_busy());
        let notified = match self.wake.unwrap().receive_until(&mut [0; 64], time::now()?) {
            Ok(_) => true,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => false,
            Err(error) => return Err(error),
        };
        if !self.refresh_due && !notified {
            return Ok(());
        }
        self.next_poll = self
            .next_poll
            .checked_add(1)
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        let deadline = time::after(100)?;
        self.pair.unwrap().0.send(&wire::encode([
            wire::POLL_TAGGED,
            self.next_poll as u64,
            0,
            0,
            0,
            0,
            0,
            0,
        ]))?;
        self.refresh_due = false;
        self.polling = Some(Pending {
            id: self.next_poll,
            deadline,
            late: false,
        });
        Ok(())
    }
    pub(in crate::service) fn progress_refresh(&mut self, world: &mut World) -> Result<(), Error> {
        let Some(p) = &mut self.polling else {
            return Ok(());
        };
        let now = time::now()?;
        let mut bytes = [0; 64];
        match self.pair.unwrap().1.receive_until(&mut bytes, now) {
            Ok(len) => {
                let words = wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?;
                if let Some(reply) = wire::untag_poll(words, p.id) {
                    if p.late {
                        time::storage_sample("matrix-poll-resumed", &[p.id as u64]);
                    }
                    self.polling = None;
                    self.inherit_reply(world, reply);
                } else {
                    time::storage_sample("matrix-poll-unmatched", &[p.id as u64, words[0] >> 32]);
                }
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => {}
            Err(error) => return Err(error),
        }
        if let Some(p) = &mut self.polling
            && !p.late
            && time::reached(now, p.deadline)
        {
            p.late = true;
            // Lateness alone loses neither authority nor input. Preserve focus
            // and ordering; actual upstream cancellation/overflow still cancels.
            #[cfg(feature = "interaction-delay")]
            for slot in 0..CHILDREN {
                // Test stimulus: wake local children to prove their queries can
                // complete while the upstream reply is deliberately held.
                world.notify(slot);
            }
            time::storage_sample("matrix-poll-delayed", &[p.id as u64]);
        }
        Ok(())
    }
}
