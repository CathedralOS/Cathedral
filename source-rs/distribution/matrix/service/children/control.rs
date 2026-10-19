//! Forwarded requests stay pending while upstream child work can progress.
//! Control is a FIFO request/reply stream the child answers in order, so an
//! unanswered request is counted and its late reply skipped rather than letting
//! it be read as the answer to a newer request. A slow child costs the asker a
//! timeout. At the unanswered-request limit, stop issuing probes rather than
//! closing the child's endpoint (which could destroy its unsaved work).
//! One request may be pending per child: the owner's own, or a loss query
//! sent before a sibling's close request, acted on when it answers.
use super::*;

/// Unanswered requests tolerated before the control link is retired.
pub(super) const MAX_STALE: u8 = 8;

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
    pub(in crate::service) fn wait(&self) -> ([Option<Handle>; CHILDREN], Option<u64>) {
        let mut handles = [None; CHILDREN];
        let mut deadline: Option<u64> = None;
        for (slot, handle) in handles.iter_mut().enumerate() {
            let Some((_, until, _)) = &self.pending[slot] else {
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
            if self.retiring[slot].is_none() {
                *handle = self.slots[slot]
                    .as_ref()
                    .and_then(|child| child.control)
                    .map(|control| control.1);
            }
            deadline = Some(match deadline {
                Some(d) if !time::reached(d, *until) => d,
                _ => *until,
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
                } else {
                    Err(abi::BUSY)
                }
            }
        };
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
            (Purpose::Close, _) => wire::error(abi::BUSY),
        };
        server.reply(&request, &wire::encode(response))?;
        Ok(retired)
    }
}
