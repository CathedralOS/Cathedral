//! Own direct children and their local connections. Pending owner RPCs leave the
//! service available for the child's upstream requests, avoiding recursive deadlock.
mod admission;
mod control;
use super::*;
use cathedral_user_runtime::{ipc::Handle, server::Request, task::Child, time};
type Pair = (Handle, Handle);
struct Owned {
    task: Child,
    reservation: Option<cathedral_user_runtime::task::Launch>,
    control: Pair,
    data: Pair,
    notify: Handle,
}
#[derive(Default)]
pub(super) struct Children {
    slots: [Option<Owned>; 2],
    next_frames: [Option<u64>; 2],
    pending: Option<(Request, usize, u64)>,
}
impl Children {
    pub(super) fn notify(&self, world: &mut World) -> Result<(), Error> {
        for (slot, child) in self.slots.iter().enumerate() {
            if world.take_notification(slot)
                && let Some(child) = child
            {
                match child.notify.send(&[]) {
                    Ok(()) => (),
                    // A queued hint already guarantees a wake. Death is handled
                    // through the data connection and structured child lifetime.
                    Err(Error(code))
                        if code == abi::WOULD_BLOCK as i64 || code == abi::PEER_CLOSED as i64 => {}
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(())
    }
    pub(super) fn dispatch(
        &mut self,
        request: &Request,
        words: wire::Packet,
        server: &mut Server,
        world: &mut World,
    ) -> Result<bool, Error> {
        let routed = words[0] >> wire::OP_BITS;
        if routed == 0
            && !matches!(
                words[0],
                wire::SPAWN | wire::RUN | wire::RUN_MEMORY | wire::STOP
            )
        {
            return Ok(false);
        }
        let result = if self.pending.is_some() {
            Err(Error(abi::BUSY as i64))
        } else if routed != 0 {
            match self.forward(words, routed) {
                Ok(slot) => {
                    self.pending = Some((*request, slot, time::after(100)?));
                    return Ok(true);
                }
                Err(error) => Err(error),
            }
        } else {
            match words {
                [wire::RUN_MEMORY, slot @ 0..=1, frames, 0, 0, 0, 0, 0] => {
                    if self.slots[slot as usize].is_some() {
                        Err(Error(abi::BUSY as i64))
                    } else {
                        self.next_frames[slot as usize] = Some(frames);
                        Ok(())
                    }
                }
                [wire::SPAWN, slot @ 0..=1, launch, loans, argument, 0, 0, 0] => {
                    self.start(slot as usize, launch, loans, argument, server, world)
                }
                [
                    wire::RUN,
                    slot @ 0..=1,
                    image,
                    argument,
                    channels,
                    descendants,
                    images,
                    cpu,
                ] => self.run(
                    slot as usize,
                    [image, argument, channels, descendants, images, cpu],
                    server,
                    world,
                ),
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
        child.notify.close()?;
        if let Some(reservation) = child.reservation {
            reservation.release()?;
            // Creator death returns descendant reservations with fresh local names.
            // Active siblings remain BUSY; static legacy grants remain DENIED.
            let mut index = 0;
            while let Ok(launch) = cathedral_user_runtime::task::Launch::at(index) {
                match launch.release() {
                    Ok(()) => (),
                    Err(Error(code)) if code == abi::BUSY as i64 || code == abi::DENIED as i64 => {
                        index += 1
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(())
    }
}
