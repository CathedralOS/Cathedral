//! Forwarded requests stay pending while upstream child work can progress.
//! Control is a FIFO request/reply stream the child answers in order, so an
//! unanswered request is counted and its late reply skipped rather than letting
//! it be read as the answer to a newer request. A slow child costs the asker a
//! timeout. At the unanswered-request limit, stop issuing probes rather than
//! closing the child's endpoint (which could destroy its unsaved work).
//! A stream that fails outright is not death either: the request stays pending
//! for a short grace so the kernel's outcome, not the broken link, decides
//! between PEER_CLOSED and BUSY.
//! One request may be pending per child: the owner's own, or a loss query
//! sent before a sibling's close request, acted on when it answers.
use super::*;

/// Unanswered requests tolerated before the control link is retired.
pub(super) const MAX_STALE: u8 = 8;
/// Ticks a request waits for the kernel outcome of a child whose control
/// stream failed before it is answered BUSY. Reclamation of an exited child
/// takes a few ticks; a live child that merely broke its link stays unknown.
const OUTCOME_GRACE_TICKS: u64 = 50;

impl Children {
    pub(super) fn forward(&mut self, mut words: wire::Packet, routed: u64) -> Result<usize, Error> {
        let hop = routed & 7;
        if !(1..=CHILDREN as u64).contains(&hop) {
            Err(Error(abi::DENIED as i64))
        } else if let Some(child) = &self.slots[(hop - 1) as usize] {
            if child.stale >= MAX_STALE {
                return Err(Error(abi::TIMED_OUT as i64));
            }
            let control = child.control.ok_or(Error(abi::PEER_CLOSED as i64))?;
            words[0] = (words[0] & 0xffff) | ((routed >> 3) << wire::OP_BITS);
            if let Err(error) = control.0.send(&wire::encode(words)) {
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
    ) -> ([Option<Handle>; CHILDREN], Option<u64>) {
        let mut handles = [None; CHILDREN];
        let mut deadline: Option<u64> = None;
        for (slot, handle) in handles.iter_mut().enumerate() {
            let Some((_, until, purpose)) = &self.pending[slot] else {
                // Late replies are drained even while nobody is asking. This
                // restores probe capacity without mistaking an old answer for
                // a fresh observation or abandoning the live child's channel.
                *handle = self.slots[slot].as_ref().and_then(|child| {
                    (child.stale > 0)
                        .then_some(child.control)
                        .flatten()
                        .map(|p| p.1)
                });
                continue;
            };
            if teardown_busy && matches!(purpose, Purpose::Close) {
                continue;
            }
            let until = if self.retiring[slot].is_none() {
                *handle = self.slots[slot]
                    .as_ref()
                    .and_then(|child| child.control)
                    .map(|control| control.1);
                *until
            } else {
                now.wrapping_add(1)
            };
            deadline = Some(match deadline {
                Some(d) if !time::reached(d, until) => d,
                _ => until,
            });
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
        let Some((request, deadline, purpose)) = self.pending[slot] else {
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
            if let Some(child) = &mut self.slots[slot]
                && child.stale > 0
                && let Some(control) = child.control
            {
                match control.1.receive_until(&mut [0; 64], time::now()?) {
                    Ok(_) => child.stale -= 1,
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                    // No late replies can arrive on a failed stream. Do not
                    // leave its permanently-ready handle in the idle wait set.
                    Err(_) => child.stale = 0,
                }
            }
            return Ok(false);
        };
        // Teardown publishes erasures on the parent stream. Leave a close's
        // answer queued until that stream is free; ordinary polling must not
        // turn a clean answer into a spurious refusal. wait() excludes this
        // receiver meanwhile so a ready reply cannot cause a busy loop.
        if matches!(purpose, Purpose::Close) && (parent.upstream_busy() || parent.storing()) {
            // The sibling may already have answered; nobody is looking. Its
            // deadline must not run out while this matrix is the one that is
            // busy, or the first poll afterwards would refuse a clean answer
            // as a timeout. Keep at least the outcome grace ahead of now.
            let grace = time::now()?.wrapping_add(OUTCOME_GRACE_TICKS);
            if let Some(pending) = &mut self.pending[slot]
                && time::reached(grace, pending.1)
            {
                pending.1 = grace;
            }
            return Ok(false);
        }
        let mut retired = false;
        let now = time::now()?;
        let mut bytes = [0; 64];
        let result = if let Some(error) = self.retiring[slot] {
            Err(Error(error))
        } else {
            match self.slots[slot].as_ref().and_then(|child| child.control) {
                Some(control) => control.1.receive_until(&mut bytes, now),
                None => Err(Error(abi::PEER_CLOSED as i64)),
            }
        };
        let answer = match result {
            Ok(len) => {
                let child = self.slots[slot].as_mut().unwrap();
                if child.stale > 0 {
                    // The answer to a request the asker already gave up on.
                    #[cfg(feature = "trace-storage")]
                    time::storage_sample(
                        "matrix-control-stale",
                        &[
                            request.incarnation,
                            slot as u64,
                            now,
                            deadline,
                            child.stale as u64,
                        ],
                    );
                    child.stale -= 1;
                    return Ok(false);
                }
                Ok(wire::decode(&bytes[..len]).unwrap_or(wire::error(abi::IO_ERROR)))
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 && !time::reached(now, deadline) => {
                return Ok(false);
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                let child = self.slots[slot].as_mut().unwrap();
                child.stale += 1;
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
            (Purpose::Close, Ok(words)) if words[0] == 0 && words[1] == 0 => {
                self.stop(slot, server, world, parent)?;
                retired = true;
                [0; 8]
            }
            (Purpose::Close, _) if retired => [0; 8],
            (Purpose::Close, answer) => {
                let (status, loss) = match answer {
                    Ok(words) => (words[0], words[1]),
                    Err(code) => (code, 0),
                };
                time::storage_sample("matrix-close-refused", &[slot as u64, status, loss]);
                wire::error(abi::BUSY)
            }
        };
        server.reply(&request, &wire::encode(response))?;
        Ok(retired)
    }
}
