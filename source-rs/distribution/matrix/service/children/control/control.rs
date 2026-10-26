// Forwarded requests stay pending while upstream child work can progress.
// Shared links retain exact request identity after timeout. Drain that late reply
// before issuing new work; repeated asks cannot replay or abandon the old request.
// Connection setup progresses while child upstream work remains serviceable.
// Incoming owner operations start in owner.rs and are selected by Omega requests/control.omg.
// A stream that fails outright is not death either: the request stays pending
// for a short grace so the kernel's outcome, not the broken link, decides
// between PEER_CLOSED and BUSY.
// One request may be pending per child: the owner's own, or a loss query
// sent before a sibling's close request, acted on when it answers.
use super::*;
use cathedral_stock_matrix::policy::lifecycle as decisions;

/// Ticks a request waits for the kernel outcome of a child whose control
/// stream failed before it is answered BUSY. Reclamation of an exited child
/// takes a few ticks; a live child that merely broke its link stays unknown.
const OUTCOME_GRACE_TICKS: u64 = 50;

impl Children {
    pub(super) fn forward(&mut self, mut words: wire::Packet, routed: u64) -> Result<usize, Error> {
        let hop = routed & 7;
        if !(1..=CHILDREN as u64).contains(&hop) {
            Err(Error(abi::DENIED as i64))
        } else if let Some(child) = &mut self.slots[(hop - 1) as usize] {
            words[0] = (words[0] & 0xffff) | ((routed >> 3) << wire::OP_BITS);
            if let Err(error) = child.control.ask(words) {
                if error.0 == abi::PEER_CLOSED as i64 && !self.completed((hop - 1) as usize)? {
                    return Err(Error(abi::BUSY as i64));
                }
                return Err(error);
            }
            Ok((hop - 1) as usize)
        } else {
            Err(Error(abi::PEER_CLOSED as i64))
        }
    }
    /// The control replies to wait for and the earliest deadline among them.
    /// A slot whose stream already failed is polled again next tick, so the
    /// child's kernel outcome is observed soon after reclamation.
    pub(in crate::service) fn wait(
        &self,
        teardown_busy: bool,
        now: u64,
    ) -> ([Option<Interest>; CHILDREN * 2], Option<u64>) {
        let mut handles = [None; CHILDREN * 2];
        let mut deadline: Option<u64> = None;
        for slot in 0..CHILDREN {
            let Some(child) = &self.slots[slot] else {
                continue;
            };
            let pending = self.pending[slot].as_ref();
            let paused = pending.is_some_and(|(_, _, purpose)| {
                decisions::wait(matches!(purpose, Purpose::Close), teardown_busy)
            });
            let (interests, opening) = child.control.wait(
                !paused
                    && self.retiring[slot].is_none()
                    && (pending.is_some() || child.control.has_late()),
            );
            handles[slot * 2..slot * 2 + 2].copy_from_slice(&interests);
            let answer = pending.filter(|_| !paused).map(|(_, until, _)| {
                if self.retiring[slot].is_some() {
                    now.wrapping_add(1)
                } else {
                    *until
                }
            });
            for until in opening.into_iter().chain(answer) {
                deadline = Some(
                    deadline.map_or(until, |d| if time::reached(d, until) { until } else { d }),
                );
            }
        }
        (handles, deadline)
    }
    pub(in crate::service) fn finish(
        &mut self,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<Option<usize>, Error> {
        let mut retired = None;
        for slot in 0..CHILDREN {
            if self.finish_slot(slot, server, world, parent)? {
                retired = Some(slot);
            }
        }
        Ok(retired)
    }
    /// Whether the child in `slot` was retired while answering.
    fn finish_slot(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<bool, Error> {
        if let Some(child) = &mut self.slots[slot] {
            child.control.progress();
            // Only a launched first app, never boot/setup providers or chrome.
            #[cfg(feature = "control-failure-probe")]
            if slot == 1
                && self.catalogue.entry(0).is_some_and(|row| row[1] != 0)
                && child.control.cut_ready()
            {
                cathedral_user_runtime::write(
                    b"Cathedral control probe: owner link cut; child retained\n",
                )?;
            }
        }
        let Some((request, mut deadline, purpose)) = self.pending[slot] else {
            self.close_paused[slot] = false;
            // A disconnect observed while a reply was pending must not be lost
            // when that reply (including a LOSS survey) completes.
            if self.slots[slot]
                .as_ref()
                .is_some_and(|child| child.disconnected)
                && !parent.upstream_busy()
                && !parent.storing()
            {
                self.stop(slot, server, world, parent)?;
                return Ok(true);
            }
            if let Some(child) = &mut self.slots[slot] {
                child.control.drain();
            }
            return Ok(false);
        };
        // Teardown publishes erasures on the parent stream. Leave a close's
        // answer queued until that stream is free; ordinary polling must not
        // turn a clean answer into a spurious refusal. wait() excludes this
        // receiver meanwhile so a ready reply cannot cause a busy loop.
        if decisions::wait(
            matches!(purpose, Purpose::Close),
            parent.upstream_busy() || parent.storing(),
        ) {
            self.close_paused[slot] = true;
            return Ok(false);
        }
        if core::mem::take(&mut self.close_paused[slot]) {
            // Extend on resumption, not on the last visit before sleeping.
            // An idle busy interval can be longer than the original deadline.
            let grace = time::now()?.wrapping_add(OUTCOME_GRACE_TICKS);
            if time::reached(grace, deadline) {
                deadline = grace;
                self.pending[slot].as_mut().unwrap().1 = grace;
            }
        }
        let mut retired = false;
        let now = time::now()?;
        let result = if let Some(error) = self.retiring[slot] {
            Err(Error(error))
        } else {
            match self.slots[slot].as_mut() {
                Some(child) => child.control.receive(now, deadline),
                None => Err(Error(abi::PEER_CLOSED as i64)),
            }
        };
        let answer = match result {
            Ok(Some(words)) => Ok(words),
            Ok(None) if !time::reached(now, deadline) => return Ok(false),
            Ok(None) => {
                self.slots[slot].as_mut().unwrap().control.expire();
                Err(abi::TIMED_OUT)
            }
            Err(error) => {
                // Defer teardown while another exchange owns the parent/provider
                // stream. A ready reply above can still complete normally.
                if parent.upstream_busy() || parent.storing() {
                    self.retiring[slot] = Some(error.0);
                    return Ok(false);
                }
                // A failed control stream is not process death. Only the kernel
                // outcome permits automatic reclamation; otherwise report busy
                // so callers cannot mistake unknown loss for consent or death.
                if self.slots[slot].is_none() {
                    // An explicit owner STOP may have won a pending close race.
                    retired = true;
                    Err(abi::PEER_CLOSED)
                } else if self.completed(slot)? {
                    self.stop(slot, server, world, parent)?;
                    retired = true;
                    Err(error.0 as u64)
                } else if !time::reached(now, deadline) {
                    // Reclamation is incremental, so the outcome usually follows
                    // the broken stream within a few ticks. Keep the request
                    // pending for a bounded grace, inside the asker's deadline.
                    // Shortening is idempotent: later visits never extend it.
                    let grace = now.wrapping_add(OUTCOME_GRACE_TICKS);
                    if let Some(pending) = &mut self.pending[slot]
                        && !time::reached(grace, pending.1)
                    {
                        pending.1 = grace;
                    }
                    self.retiring[slot] = Some(error.0);
                    return Ok(false);
                } else {
                    Err(abi::BUSY)
                }
            }
        };
        #[cfg(feature = "trace-storage")]
        if matches!(purpose, Purpose::Loss | Purpose::Close) {
            time::storage_sample(
                "matrix-control-answer",
                &[
                    request.incarnation,
                    slot as u64,
                    u64::from(matches!(purpose, Purpose::Close)),
                    now,
                    deadline,
                    answer.as_ref().map_or_else(|code| *code, |words| words[0]),
                ],
            );
        }
        self.pending[slot] = None;
        self.retiring[slot] = None;
        if matches!(purpose, Purpose::Loss) {
            self.loss_answer(slot, answer, server)?;
            return Ok(retired);
        }
        let response = match (purpose, answer) {
            (Purpose::Loss, _) => unreachable!("survey answered above"),
            (Purpose::Owner, Ok(words)) => words,
            (Purpose::Owner, Err(code)) => wire::error(code),
            // The sibling goes only if it answered and reported nothing it would lose.
            (Purpose::Close, answer) => match decisions::close(retired, answer) {
                decisions::Close::Stop => {
                    self.stop(slot, server, world, parent)?;
                    retired = true;
                    [0; 8]
                }
                decisions::Close::AlreadyStopped => [0; 8],
                decisions::Close::Refuse => {
                    let (status, loss) = match answer {
                        Ok(words) => (words[0], words[1]),
                        Err(code) => (code, 0),
                    };
                    time::storage_sample("matrix-close-refused", &[slot as u64, status, loss]);
                    wire::error(abi::BUSY)
                }
            },
        };
        server.reply(&request, &wire::encode(response))?;
        Ok(retired)
    }
}
