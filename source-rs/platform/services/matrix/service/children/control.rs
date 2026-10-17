//! Forwarded owner requests stay pending while upstream child work can progress.
//! Control is a FIFO request/reply stream the child answers in order, so an
//! unanswered request is counted and its late reply skipped rather than letting
//! it be read as the answer to a newer request. A slow child costs the owner a
//! timeout; only a child that stops answering altogether loses its control link.
use super::*;

/// Unanswered owner requests tolerated before the control link is retired.
const MAX_STALE: u8 = 8;

impl Children {
    pub(super) fn forward(&self, mut words: wire::Packet, routed: u64) -> Result<usize, Error> {
        let hop = routed & 3;
        if !(1..=2).contains(&hop) {
            Err(Error(abi::DENIED as i64))
        } else if let Some(child) = &self.slots[(hop - 1) as usize] {
            let control = child.control.ok_or(Error(abi::PEER_CLOSED as i64))?;
            words[0] = (words[0] & 0xffff) | ((routed >> 2) << wire::OP_BITS);
            control.0.send(&wire::encode(words))?;
            Ok((hop - 1) as usize)
        } else {
            Err(Error(abi::PEER_CLOSED as i64))
        }
    }
    pub(in crate::service) fn wait(&self) -> (Option<Handle>, Option<u64>) {
        match &self.pending {
            Some((_, slot, deadline)) => (
                if self.retiring.is_some() {
                    None
                } else {
                    self.slots[*slot]
                        .as_ref()
                        .and_then(|child| child.control)
                        .map(|control| control.1)
                },
                Some(*deadline),
            ),
            None => (None, None),
        }
    }
    pub(in crate::service) fn finish(
        &mut self,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<Option<usize>, Error> {
        let Some((request, slot, deadline)) = self.pending else {
            return Ok(None);
        };
        let mut retired = None;
        let now = time::now()?;
        let mut bytes = [0; 64];
        let result = if let Some(error) = self.retiring {
            Err(Error(error))
        } else {
            match self.slots[slot].as_ref().and_then(|child| child.control) {
                Some(control) => control.1.receive_until(&mut bytes, now),
                None => Err(Error(abi::PEER_CLOSED as i64)),
            }
        };
        let response = match result {
            Ok(len) => {
                let child = self.slots[slot].as_mut().unwrap();
                if child.stale > 0 {
                    // The answer to a request the owner already gave up on.
                    child.stale -= 1;
                    return Ok(None);
                }
                wire::decode(&bytes[..len]).unwrap_or(wire::error(abi::IO_ERROR))
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 && !time::reached(now, deadline) => {
                return Ok(None);
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                let child = self.slots[slot].as_mut().unwrap();
                child.stale += 1;
                if child.stale >= MAX_STALE
                    && let Some((send, receive)) = child.control.take()
                {
                    send.close()?;
                    receive.close()?;
                }
                wire::error(abi::TIMED_OUT)
            }
            Err(error) => {
                // Defer teardown while another exchange owns the parent/provider
                // stream. A ready owner reply above can still complete normally.
                if parent.upstream_busy() || parent.storing() {
                    self.retiring = Some(error.0);
                    return Ok(None);
                }
                // A dead child cannot answer; reclaim it and its subtree.
                self.stop(slot, server, world, parent)?;
                retired = Some(slot);
                wire::error(error.0 as u64)
            }
        };
        self.pending = None;
        server.reply(&request, &wire::encode(response))?;
        Ok(retired)
    }
}
