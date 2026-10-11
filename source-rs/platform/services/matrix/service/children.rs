//! Own direct children and their local connections. Pending owner RPCs leave the
//! service available for the child's upstream requests, avoiding recursive deadlock.
mod admission;
use super::*;
use cathedral_user_runtime::{ipc::Handle, server::Request, task::Child, time};
type Pair = (Handle, Handle);
struct Owned {
    task: Child,
    control: Pair,
    data: Pair,
}
#[derive(Default)]
pub(super) struct Children {
    slots: [Option<Owned>; 2],
    pending: Option<(Request, usize, u64)>,
}
impl Children {
    pub(super) fn dispatch(
        &mut self,
        request: &Request,
        mut words: wire::Packet,
        server: &mut Server,
        world: &mut World,
    ) -> Result<bool, Error> {
        let routed = words[0] >> wire::OP_BITS;
        if routed == 0 && !matches!(words[0], wire::SPAWN | wire::STOP) {
            return Ok(false);
        }
        let result = if self.pending.is_some() {
            Err(Error(abi::BUSY as i64))
        } else if routed != 0 {
            let hop = routed & 3;
            if !(1..=2).contains(&hop) {
                Err(Error(abi::DENIED as i64))
            } else if let Some(child) = &self.slots[(hop - 1) as usize] {
                words[0] = (words[0] & 0xffff) | ((routed >> 2) << wire::OP_BITS);
                match child.control.0.send(&wire::encode(words)) {
                    Ok(()) => {
                        self.pending = Some((*request, (hop - 1) as usize, time::after(100)?));
                        return Ok(true);
                    }
                    Err(error) => Err(error),
                }
            } else {
                Err(Error(abi::PEER_CLOSED as i64))
            }
        } else {
            match words {
                [wire::SPAWN, slot @ 0..=1, launch, loans, argument, 0, 0, 0] => {
                    self.start(slot as usize, launch, loans, argument, server, world)
                }
                [wire::STOP, slot @ 0..=1, 0, 0, 0, 0, 0, 0] => {
                    self.stop(slot as usize, server, world)
                }
                _ => Err(Error(abi::DENIED as i64)),
            }
        };
        server.reply(
            request,
            &wire::encode(wire::error(result.err().map_or(0, |e| e.0 as u64))),
        )?;
        Ok(true)
    }
    pub(super) fn wait(&self) -> (Option<Handle>, Option<u64>) {
        match &self.pending {
            Some((_, slot, deadline)) => (
                Some(self.slots[*slot].as_ref().unwrap().control.1),
                Some(*deadline),
            ),
            None => (None, None),
        }
    }
    pub(super) fn finish(
        &mut self,
        server: &mut Server,
        world: &mut World,
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
                self.stop(slot, server, world)?;
                retired = Some(slot);
                wire::error(error.0 as u64)
            }
        };
        self.pending = None;
        server.reply(&request, &wire::encode(response))?;
        Ok(retired)
    }
    pub(super) fn disconnected(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        if self.pending.is_some_and(|(_, pending, _)| pending == slot) {
            world.disconnected(slot);
            return Ok(());
        }
        if self.slots[slot].is_some() {
            self.stop(slot, server, world)?;
        } else {
            world.disconnected(slot);
        }
        Ok(())
    }
    fn stop(&mut self, slot: usize, server: &mut Server, world: &mut World) -> Result<(), Error> {
        let child = self.slots[slot]
            .take()
            .ok_or(Error(abi::PEER_CLOSED as i64))?;
        server.remove(slot);
        world.disconnected(slot);
        child.task.cancel()?;
        child.task.wait()?;
        child.data.0.close()?;
        child.data.1.close()?;
        Ok(())
    }
}
