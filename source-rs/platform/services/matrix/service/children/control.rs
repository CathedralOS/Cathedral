//! Forwarded owner requests stay pending while upstream child work can progress.
//! A failed/timed-out stream is retired before another request can reuse it.
use super::*;

impl Children {
    pub(super) fn forward(&self, mut words: wire::Packet, routed: u64) -> Result<usize, Error> {
        let hop = routed & 3;
        if !(1..=2).contains(&hop) {
            Err(Error(abi::DENIED as i64))
        } else if let Some(child) = &self.slots[(hop - 1) as usize] {
            words[0] = (words[0] & 0xffff) | ((routed >> 2) << wire::OP_BITS);
            child.control.0.send(&wire::encode(words))?;
            Ok((hop - 1) as usize)
        } else {
            Err(Error(abi::PEER_CLOSED as i64))
        }
    }
    pub(in crate::service) fn wait(&self) -> (Option<Handle>, Option<u64>) {
        match &self.pending {
            Some((_, slot, deadline)) => (
                Some(self.slots[*slot].as_ref().unwrap().control.1),
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
        let result = self.slots[slot]
            .as_ref()
            .unwrap()
            .control
            .1
            .receive_until(&mut bytes, now);
        let response = match result {
            Ok(len) => wire::decode(&bytes[..len]).unwrap_or(wire::error(abi::IO_ERROR)),
            Err(Error(code)) if code == abi::TIMED_OUT as i64 && !time::reached(now, deadline) => {
                return Ok(None);
            }
            Err(error) => {
                // Do not reuse a stream that may still produce an uncorrelated reply.
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
