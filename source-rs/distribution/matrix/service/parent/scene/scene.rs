// Launch visibility transaction: prepare without mutation, send one step, await
// its exact stream owner, then commit or abort. blocking.rs handles selection and teardown.
use super::*;
use cathedral_stock_matrix::policy::operation::Ticket;
pub(in crate::service::parent) struct Pending {
    ticket: Ticket,
    revision: u64,
    count: usize,
    step: usize, // begin, surfaces, commit, abort
    status: u64,
    pub(in crate::service::parent) waiting: bool,
    pub(in crate::service::parent) reply: Option<u64>,
    deadline: u64,
    late: bool,
    retry: Option<u64>,
    #[cfg(feature = "scene-delay")]
    hold: Option<u64>,
}
pub(in crate::service) struct Completed {
    pub ticket: Ticket,
    pub status: u64,
    pub committed: bool,
}
impl Parent {
    pub(in crate::service) fn scene_active(&self) -> bool {
        self.scene.is_some()
    }
    pub(in crate::service) fn scene_commit_ready(&self) -> bool {
        self.scene
            .as_ref()
            .is_some_and(|p| !p.waiting && p.step == p.count + 1)
    }
    pub(in crate::service) fn cancel_scene(&mut self, status: u64) {
        if let Some(p) = &mut self.scene
            && p.status == 0
        {
            p.status = status;
            time::storage_sample("launch-scene-cancel", &[p.ticket.raw(), status]);
        }
    }
    pub(in crate::service) fn scene_wait(&self) -> Option<(Handle, Option<u64>)> {
        self.scene.as_ref().map(|p| {
            let until = p.retry.or_else(|| (!p.late).then_some(p.deadline));
            #[cfg(feature = "scene-delay")]
            let until = p.hold.filter(|until| *until != 0).or(until);
            (self.output().unwrap().1, until)
        })
    }
    pub(in crate::service) fn begin_scene(
        &mut self,
        ticket: Ticket,
        world: &World,
        previous: Option<usize>,
        next: usize,
    ) -> Result<(), Error> {
        assert!(!self.publishing());
        self.output()?;
        let mut count = 0;
        for slot in 1..BACKGROUND {
            if Some(slot) != previous && slot != next {
                continue;
            }
            for rank in 0..wire::MAX_SURFACES {
                if let Some(words) = world.preview_replay(slot, rank, slot == next) {
                    if count == self.scene_packets.len() {
                        return Err(Error(abi::NO_MEMORY as i64));
                    }
                    self.scene_packets[count] = words;
                    count += 1;
                }
            }
        }
        self.next_revision = self
            .next_revision
            .checked_add(1)
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        self.scene = Some(Pending {
            ticket,
            revision: self.next_revision,
            count,
            step: 0,
            status: 0,
            waiting: false,
            reply: None,
            deadline: time::after(300)?,
            late: false,
            retry: None,
            #[cfg(feature = "scene-delay")]
            hold: if next == 2 && !self.scene_probe_used {
                self.scene_probe_used = true;
                Some(0) // Arm the gate only after staging actually completes.
            } else {
                None
            },
        });
        Ok(())
    }
    pub(in crate::service) fn progress_scene(&mut self) -> Result<Option<Completed>, Error> {
        let Some(p) = &self.scene else {
            return Ok(None);
        };
        if !p.waiting {
            return self.send_scene();
        }
        let status = if self.upstream || self.scene.as_ref().unwrap().reply.is_some() {
            self.scene.as_mut().unwrap().reply.take()
        } else {
            let mut bytes = [0; 64];
            match self.output()?.1.receive_until(&mut bytes, time::now()?) {
                Ok(len) => Some(
                    cathedral_platform_contracts::display::decode(&bytes[..len])
                        .ok_or(Error(abi::IO_ERROR as i64))?[0],
                ),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => None,
                Err(e) => return Err(e),
            }
        };
        let p = self.scene.as_mut().unwrap();
        let Some(status) = status else {
            if !p.late && time::reached(time::now()?, p.deadline) {
                p.late = true;
                time::storage_sample("launch-scene-delayed", &[p.ticket.raw()]);
            }
            return Ok(None);
        };
        #[cfg(feature = "scene-commit-hold")]
        if p.step == p.count + 1
            && status == 0
            && let Some(until) = p.hold
        {
            let now = time::now()?;
            if until == 0 {
                p.hold = Some(time::after(300)?);
                p.reply = Some(status);
                time::storage_sample("launch-scene-held", &[p.ticket.raw()]);
                return Ok(None);
            }
            if !time::reached(now, until) {
                p.reply = Some(status);
                return Ok(None);
            }
            p.hold = None;
            time::storage_sample("launch-scene-released", &[p.ticket.raw()]);
        }
        p.waiting = false;
        if p.step == p.count + 2 {
            // Failure to abort leaves upstream ownership unknown: fail closed.
            if status != 0 {
                return Err(Error(status as i64));
            }
            return Ok(Some(self.finish_scene(false)));
        }
        if p.step == p.count + 1 && status == 0 {
            // Cancellation after COMMIT was sent cannot undo its acceptance.
            // The launch owner restores the previous scene before rollback.
            return Ok(Some(self.finish_scene(true)));
        }
        if status != 0 {
            time::storage_sample("launch-scene-refused", &[p.step as u64, status]);
            if p.status == 0 {
                p.status = status;
            }
            if p.step == 0 {
                return Ok(Some(self.finish_scene(false)));
            }
        }
        if p.status != 0 {
            p.step = p.count + 2;
        } else {
            p.step += 1;
        }
        // Re-enter the service loop before COMMIT so custody is checked there.
        p.retry = Some(time::now()?);
        Ok(None)
    }
    fn finish_scene(&mut self, committed: bool) -> Completed {
        let p = self.scene.take().unwrap();
        time::storage_sample(
            "launch-scene-complete",
            &[p.ticket.raw(), p.status, u64::from(committed)],
        );
        Completed {
            ticket: p.ticket,
            status: p.status,
            committed,
        }
    }
}
