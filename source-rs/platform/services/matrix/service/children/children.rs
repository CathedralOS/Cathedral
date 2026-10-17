// Own direct children and their local connections. Pending owner RPCs leave the
// service available for the child's upstream requests, avoiding recursive deadlock.
// A child may ask for a sibling: the host reads the program out of the child's
// realm and admits it into the slot the owner opened with a standing policy,
// so a launcher is a broker that never sits in what it launched.
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
/// What the owner lets a child's launch put into a slot: the child's pages,
/// channels, descendant slots, CPU ceiling and surfaces, and where it goes.
#[derive(Clone, Copy)]
struct Standing {
    pages: [u64; 2],
    channels: u64,
    descendants: u64,
    cpu: u64,
    surfaces: u64,
    place: Option<(Rect, u64)>,
}
/// Why a request was forwarded to a child, so its reply is read right: the
/// owner's own request, or a loss query before a sibling's close request.
#[derive(Clone, Copy)]
pub(super) enum Purpose {
    Owner,
    Close,
    Loss,
}
#[derive(Default)]
pub(super) struct Children {
    pub separate_storage: bool,
    slots: [Option<Owned>; 2],
    next_frames: [Option<u64>; 2],
    next_pages: [Option<[u64; 2]>; 2],
    standing: [Option<Standing>; 2],
    pending: [Option<(Request, u64, Purpose)>; 2],
    retiring: [Option<i64>; 2],
    loss: Option<loss::Survey>,
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
        if words == [wire::LOSS, 0, 0, 0, 0, 0, 0, 0] {
            self.survey_loss(*request, parent, server)?;
            return Ok(true);
        }
        if routed == 0
            && !matches!(
                words[0],
                wire::SPAWN
                    | wire::RUN
                    | wire::RUN_OBJECT
                    | wire::RUN_MEMORY
                    | wire::RUN_PAGES
                    | wire::LAUNCH_POLICY
                    | wire::LAUNCH_PLACE
                    | wire::STOP
            )
        {
            return Ok(false);
        }
        if self.loss.is_some() {
            server.reply(request, &wire::encode(wire::error(abi::BUSY)))?;
            return Ok(true);
        }
        // A program from the realm is read first; the owner is answered when
        // the child is admitted, from launch_object.
        if routed == 0
            && let [
                wire::RUN_OBJECT,
                slot @ 0..=1,
                object,
                scope,
                argument,
                channels,
                descendants,
                cpu,
            ] = words
        {
            let slot = slot as usize;
            if parent.storing() || self.slots[slot].is_some() || !server.available(slot) {
                server.reply(request, &wire::encode(wire::error(abi::BUSY)))?;
            } else if object == 0 {
                server.reply(request, &wire::encode(wire::error(abi::INVALID_ARGUMENT)))?;
            } else {
                parent.storage_launch(
                    *request,
                    parent::LaunchRequest {
                        slot,
                        object,
                        scope,
                        argument,
                        channels,
                        descendants,
                        cpu,
                        pages: None,
                        stamp: 0,
                    },
                )?;
            }
            return Ok(true);
        }
        let result = if routed != 0 {
            let hop = (routed & 3) as usize;
            if (1..=2).contains(&hop) && self.pending[hop - 1].is_some() {
                Err(Error(abi::BUSY as i64))
            } else {
                match self.forward(words, routed) {
                    Ok(slot) => {
                        self.pending[slot] = Some((*request, time::after(100)?, Purpose::Owner));
                        self.retiring[slot] = None;
                        return Ok(true);
                    }
                    Err(error) => Err(error),
                }
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
                [
                    wire::LAUNCH_POLICY,
                    slot @ 0..=1,
                    private,
                    readers,
                    channels,
                    descendants,
                    cpu,
                    surfaces,
                ] if private <= cathedral_kernel_contracts::memory::MAX_PAGES as u64
                    && readers <= cathedral_kernel_contracts::memory::MAX_PAGES as u64
                    && surfaces <= wire::MAX_SURFACES as u64 =>
                {
                    let place = self.standing[slot as usize].and_then(|s| s.place);
                    self.standing[slot as usize] = Some(Standing {
                        pages: [private, readers],
                        channels,
                        descendants,
                        cpu,
                        surfaces,
                        place,
                    });
                    Ok(())
                }
                [
                    wire::LAUNCH_PLACE,
                    slot @ 0..=1,
                    x,
                    y,
                    width,
                    height,
                    keys,
                    0,
                ] => match &mut self.standing[slot as usize] {
                    Some(standing) => {
                        standing.place = Some((
                            Rect {
                                x,
                                y,
                                width,
                                height,
                            },
                            keys,
                        ));
                        Ok(())
                    }
                    None => Err(Error(abi::DENIED as i64)),
                },
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
    /// A child's request for a sibling. The other slot must be free and opened
    /// by the owner; the program is read with the requester's own scope, so a
    /// launcher can only launch what it can see.
    pub(in crate::service) fn launch(
        &mut self,
        request: Request,
        object: u64,
        parent: &mut Parent,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.loss.is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let requester = request.client.unwrap();
        let slot = 1 - requester;
        let refusal = if parent.storing() || self.slots[slot].is_some() || !server.available(slot) {
            Some(abi::BUSY)
        } else if object == 0 {
            Some(abi::INVALID_ARGUMENT)
        } else {
            None
        };
        if let Some(code) = refusal {
            return server.reply(&request, &wire::encode(wire::error(code)));
        }
        let (Some(standing), Some((_, stamp))) = (
            self.standing[slot],
            parent.storage_binding(server, requester),
        ) else {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        };
        parent.storage_launch(
            request,
            parent::LaunchRequest {
                slot,
                object,
                scope: object,
                argument: 0,
                channels: standing.channels,
                descendants: standing.descendants,
                cpu: standing.cpu,
                pages: Some(standing.pages),
                stamp,
            },
        )
    }
    /// A child's request to stop its sibling. The sibling is asked first and
    /// kept if it reports work it would lose or does not answer.
    pub(in crate::service) fn close(
        &mut self,
        request: Request,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.loss.is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let target = 1 - request.client.unwrap();
        // A reply the requester sent just before asking may still be queued.
        self.finish(server, world, parent)?;
        if self.pending[target].is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let Some(child) = self.slots[target].as_ref() else {
            return server.reply(&request, &wire::encode(wire::error(abi::PEER_CLOSED)));
        };
        if child.stale >= control::MAX_STALE {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let Some(control) = child.control else {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        };
        control
            .0
            .send(&wire::encode([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]))?;
        self.pending[target] = Some((request, time::after(100)?, Purpose::Close));
        self.retiring[target] = None;
        Ok(())
    }
    /// Admit a program the matrix just read from its realm, grant the child
    /// that object's subtree, and answer whoever asked. A child launched for a
    /// sibling also gets the slot's standing surfaces and placement. A child
    /// that cannot be granted its realm is stopped rather than left running
    /// without one.
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
            Err(code) => Err(code),
            Ok(object) => {
                let admitted = self.run_object(plan, &object, server, world);
                drop(object); // The kernel holds its own reference to the image.
                match admitted {
                    Err(error) => Err(error.0 as u64),
                    Ok(()) => match parent.storage_grant(server, plan.slot, 0, plan.scope)? {
                        Ok(()) => Ok(()),
                        Err(code) => {
                            self.stop(plan.slot, server, world, parent)?;
                            Err(code)
                        }
                    },
                }
            }
        };
        if status.is_ok()
            && request.client.is_some()
            && let Some(standing) = self.standing[plan.slot]
        {
            let _ = world.surface_limit(plan.slot, standing.surfaces as usize);
            if let Some((rect, keys)) = standing.place {
                let _ = world.place(plan.slot, rect, keys);
            }
        }
        super::requests::flush(parent, world)?;
        super::requests::replay(parent, world, plan.slot)?;
        self.notify(world)?;
        // The owner named the slot; a child learns which one it got.
        let reply = match (status, request.client) {
            (Ok(()), Some(_)) => [0, plan.slot as u64, 0, 0, 0, 0, 0, 0],
            (Ok(()), None) => [0; 8],
            (Err(code), _) => wire::error(code),
        };
        server.reply(&request, &wire::encode(reply))
    }
    pub(super) fn disconnected(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<(), Error> {
        parent.storage_close(slot)?;
        if self.pending[slot].is_some() {
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
    pub(super) fn stop(
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
