//! One identified upstream snapshot, progressed by the service loop. Lateness
//! keeps the request alive; unrelated replies cannot alter grants or input.
use super::*;
pub(super) struct Pending {
    pub(super) id: u32,
    pub(super) reply: Option<wire::Packet>,
    deadline: u64,
    late: bool,
}
impl Parent {
    pub(in crate::service) fn refresh_ready(&self) -> bool {
        self.polling.as_ref().is_some_and(|p| p.reply.is_some())
    }
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
            .filter(|p| p.reply.is_none())
            .map(|p| (self.pair.unwrap().1, (!p.late).then_some(p.deadline)))
    }
    pub fn refresh_notified(&mut self, world: &mut World) -> Result<(), Error> {
        if !self.upstream || self.wake.is_none() || self.pair.is_none() || !world.input_ready() {
            return Ok(());
        }
        assert!(self.can_poll());
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
            .filter(|id| *id <= cathedral_stock_matrix::transport::LAST_POLL)
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        let deadline = time::after(100)?;
        match self.pair.unwrap().0.send(&wire::encode([
            wire::POLL_TAGGED,
            self.next_poll as u64,
            0,
            0,
            0,
            0,
            0,
            0,
        ])) {
            Ok(()) => (),
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                self.refresh_due = true;
                self.poll_retry = Some(time::after(1)?);
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        self.refresh_due = false;
        self.polling = Some(Pending {
            id: self.next_poll,
            reply: None,
            deadline,
            late: false,
        });
        Ok(())
    }
    pub(in crate::service) fn progress_refresh(&mut self, world: &mut World) -> Result<(), Error> {
        let now = time::now()?;
        if let Some(reply) = self.polling.as_ref().and_then(|p| p.reply) {
            // A view replacement can invalidate the surface arena. Retain that
            // exact snapshot until the outstanding scene commits or aborts.
            if self.publishing() && !world.poll_preserves_view(reply) {
                return Ok(());
            }
            let p = self.polling.take().unwrap();
            if p.late {
                time::storage_sample("matrix-poll-resumed", &[p.id as u64]);
            }
            #[cfg(feature = "interaction-delay")]
            if self.publishing() && wire::poll_event(reply)[2] == wire::KEY {
                time::storage_sample(
                    "matrix-input-during-present",
                    &[self.storage_realm, p.id as u64, wire::poll_event(reply)[3]],
                );
            }
            self.inherit_reply(world, reply);
            // Malformed upstream input must fail closed, never let publication
            // proceed against revoked local bindings.
            if self.pending.as_ref().is_some_and(|p| {
                !world.binding_matches(
                    p.request.client.unwrap(),
                    p.request.incarnation,
                    self.batches[p.request.client.unwrap()].epoch,
                )
            }) {
                return Err(Error(abi::DENIED as i64));
            }
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
