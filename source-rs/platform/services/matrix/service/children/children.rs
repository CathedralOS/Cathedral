// Own direct children and their local connections. Pending owner RPCs leave the
// service available for the child's upstream requests, avoiding recursive deadlock.
use super::*;
use cathedral_user_runtime::{ipc::Handle, server::Request, task::Child, time};
type Pair = (Handle, Handle);
struct Owned {
    task: Child,
    reservation: Option<cathedral_user_runtime::task::Launch>,
    control: Option<Pair>,
    stale: u8,
    data: Pair,
    storage: Option<Pair>,
    notify: Handle,
}
#[derive(Default)]
pub(super) struct Children {
    pub separate_storage: bool,
    slots: [Option<Owned>; 2],
    next_frames: [Option<u64>; 2],
    next_pages: [Option<[u64; 2]>; 2],
    pending: Option<(Request, usize, u64)>,
    retiring: Option<i64>,
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
        parent: &mut Parent,
        words: wire::Packet,
        server: &mut Server,
        world: &mut World,
    ) -> Result<bool, Error> {
        let routed = words[0] >> wire::OP_BITS;
        if routed == 0
            && !matches!(
                words[0],
                wire::SPAWN
                    | wire::RUN
                    | wire::RUN_OBJECT
                    | wire::RUN_MEMORY
                    | wire::RUN_PAGES
                    | wire::STOP
            )
        {
            return Ok(false);
        }
        // A program from the realm is read first; the owner is answered when
        // the child is admitted, from launch_object.
        if routed == 0
            && let [
                wire::RUN_OBJECT,
                slot @ 0..=1,
                object,
                role,
                argument,
                channels,
                descendants,
                cpu,
            ] = words
        {
            let slot = slot as usize;
            if self.pending.is_some() || self.slots[slot].is_some() || !server.available(slot) {
                server.reply(request, &wire::encode(wire::error(abi::BUSY)))?;
            } else if object == 0 {
                server.reply(request, &wire::encode(wire::error(abi::INVALID_ARGUMENT)))?;
            } else {
                parent.storage_launch(
                    *request,
                    parent::LaunchRequest {
                        slot,
                        object,
                        role,
                        argument,
                        channels,
                        descendants,
                        cpu,
                    },
                )?;
            }
            return Ok(true);
        }
        let result = if self.pending.is_some() {
            Err(Error(abi::BUSY as i64))
        } else if routed != 0 {
            match self.forward(words, routed) {
                Ok(slot) => {
                    self.pending = Some((*request, slot, time::after(100)?));
                    self.retiring = None;
                    return Ok(true);
                }
                Err(error) => Err(error),
            }
        } else {
            match words {
                [wire::RUN_PAGES, slot @ 0..=1, private, readers, 0, 0, 0, 0]
                    if private <= cathedral_kernel_contracts::memory::MAX_PAGES as u64
                        && readers <= cathedral_kernel_contracts::memory::MAX_PAGES as u64 =>
                {
                    if self.slots[slot as usize].is_some() {
                        Err(Error(abi::BUSY as i64))
                    } else {
                        self.next_pages[slot as usize] = Some([private, readers]);
                        Ok(())
                    }
                }
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
                    self.stop(slot as usize, server, world, parent)
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
    /// Admit a program the matrix just read from its realm, grant the child
    /// that object's subtree, and answer the owner. A child that cannot be
    /// granted its realm is stopped rather than left running without one.
    pub(in crate::service) fn launch_object(
        &mut self,
        launched: parent::Launched,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let parent::Launched {
            plan,
            request,
            result,
        } = launched;
        let status = match result {
            Err(code) => Err(Error(code as i64)),
            Ok(object) => {
                let admitted = self.run_object(plan, &object, server, world);
                drop(object); // The kernel holds its own reference to the image.
                match admitted {
                    Err(error) => Err(error),
                    Ok(()) => match parent.storage_grant(server, plan.slot, 0, plan.object)? {
                        Ok(()) => Ok(()),
                        Err(code) => {
                            self.stop(plan.slot, server, world, parent)?;
                            Err(Error(code as i64))
                        }
                    },
                }
            }
        };
        super::requests::flush(parent, world)?;
        server.reply(
            &request,
            &wire::encode(wire::error(status.err().map_or(0, |e| e.0 as u64))),
        )
    }
    pub(super) fn disconnected(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<(), Error> {
        parent.storage_close(slot)?;
        if self.pending.is_some_and(|(_, pending, _)| pending == slot) {
            parent.batches[slot] = Default::default();
            world.disconnected(slot);
            return Ok(());
        }
        if self.slots[slot].is_some() {
            self.stop(slot, server, world, parent)?;
        } else {
            parent.batches[slot] = Default::default();
            world.disconnected(slot);
        }
        Ok(())
    }
    fn stop(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<(), Error> {
        parent.storage_close(slot)?;
        let child = self.slots[slot]
            .take()
            .ok_or(Error(abi::PEER_CLOSED as i64))?;
        server.remove(slot);
        if child.storage.is_some() {
            server.remove(slot + 3);
        }
        world.disconnected(slot);
        // Release retained upstream image readers before returning descendant funding.
        super::requests::flush(parent, world)?;
        child.task.cancel()?;
        child.task.wait()?;
        child.data.0.close()?;
        child.data.1.close()?;
        child.notify.close()?;
        if let Some((input, output)) = child.storage {
            input.close()?;
            output.close()?;
        }
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
