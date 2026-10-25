// Own direct children and their local connections. Pending owner RPCs leave the
// service available for the child's upstream requests, avoiding recursive deadlock.
// A child may ask for a sibling: the host reads the program out of the child's
// realm and admits it into the slot the owner opened with a standing policy,
// so a launcher is a broker that never sits in what it launched.
use super::*;
use cathedral_user_runtime::{ipc::Handle, server::Request, task::Child, time};
type Pair = (Handle, Handle);
struct Owned {
    funding: Option<cathedral_matrix::funding::Grant>,
    services: Option<cathedral_user_runtime::services::Offer>,
    task: Child,
    outcome: Option<cathedral_user_runtime::task::Outcome>,
    disconnected: bool,
    reservation: Option<cathedral_user_runtime::task::Launch>,
    control: Option<Pair>,
    stale: u8,
    data: Pair,
    storage: Option<Pair>,
    producer: Option<Pair>,
    // None: legacy connection. Some(false): offered but not activated.
    // Some(true): activated once, even if subsequently closed/revoked. Preserve
    // this state so interaction cannot become a fallback production endpoint.
    producer_started: Option<bool>,
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
    desktop: bool,
    requested_surfaces: [Option<u64>; CHILDREN],
    login: bool,
    entered: bool,
    startup_recovery: startup::Recovery,
    active: Option<usize>,
    running: [Option<(u64, u64)>; CHILDREN], // local ticket, program object
    next_ticket: u64,
    slots: [Option<Owned>; CHILDREN],
    saved_storage: [Option<(usize, u64)>; CHILDREN],
    next_frames: [Option<u64>; CHILDREN],
    next_pages: [Option<[u64; 2]>; CHILDREN],
    standing: [Option<Standing>; CHILDREN],
    allowed_imports: [Option<u64>; CHILDREN],
    allowed_images: [u64; CHILDREN],
    pending: [Option<(Request, u64, Purpose)>; CHILDREN],
    retiring: [Option<i64>; CHILDREN],
    /// Test-only: a close whose LOSS waits for the storage hold to end, so the
    /// sibling's clean answer lands after the busy window (delay_probe).
    #[cfg(feature = "interaction-delay")]
    held_loss: [bool; CHILDREN],
    loss: Option<loss::Survey>,
    launcher: Option<recovery::Launcher>,
    recovery_tickets: cathedral_stock_matrix::policy::recovery::Tickets,
    fallback: Option<recovery::Fallback>,
    pub(in crate::service) observations: cathedral_matrix::world::launches::Launches<CHILDREN>,
    management: cathedral_matrix::world::management::Management<CHILDREN>,
}
impl Children {
    /// Remember only successfully applied owner policy. Revocation must remain
    /// revoked when a later provider replacement restores the other children.
    pub(super) fn remember_storage(&mut self, words: wire::Packet) {
        match words {
            [
                wire::STORAGE_GRANT,
                slot @ 0..=LAST_CHILD,
                source,
                scope,
                0,
                0,
                0,
                0,
            ] => {
                self.saved_storage[slot as usize] = Some((source as usize, scope));
            }
            [
                wire::STORAGE_REVOKE,
                slot @ 0..=LAST_CHILD,
                0,
                0,
                0,
                0,
                0,
                0,
            ] => {
                self.saved_storage[slot as usize] = None;
            }
            _ => {}
        }
    }
    pub(super) fn notify(&self, world: &mut World) -> Result<(), Error> {
        for (slot, child) in self.slots.iter().enumerate() {
            if child
                .as_ref()
                .and_then(|c| c.services.as_ref())
                .is_some_and(|offer| !offer.interaction_ready())
            {
                continue;
            }
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
                    | wire::LAUNCH_IMPORTS
                    | wire::CONFIGURE
                    | wire::RESTORE_STORAGE
                    | wire::STOP
                    | wire::MANAGEMENT_GRANT
                    | wire::LAUNCH_IMAGES
                    | wire::LAUNCHER_FALLBACK
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
                slot @ 0..=LAST_CHILD,
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
                // The owner running this realm's Login over its Users in slot
                // zero is the picker. Admit its chosen user as a nested desktop
                // does, retiring the picker first so the user's first grant is
                // final. The picker's recovery stays with the owner.
                if !parent.upstream && slot == 0 && scope != 0 {
                    let named = parent.local_storage(|realm| {
                        realm.bind().map_err(|c| Error(c as i64))?;
                        Ok((realm.lookup(b"Login").ok(), realm.lookup(b"Users").ok()))
                    });
                    if let Ok((Some(login), Some(users))) = named
                        && login == object
                        && users == scope
                    {
                        self.login = true;
                        self.entered = false;
                        self.startup_recovery.owner_recovers = true;
                    }
                }
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
                        imports: self.allowed_imports[slot]
                            .unwrap_or(cathedral_platform_contracts::imports::ALL),
                    },
                )?;
            }
            return Ok(true);
        }
        let result = if routed != 0 {
            let hop = (routed & 7) as usize;
            if (1..=CHILDREN).contains(&hop) && self.pending[hop - 1].is_some() {
                Err(Error(abi::BUSY as i64))
            } else {
                match self.forward(words, routed) {
                    Ok(slot) => {
                        self.pending[slot] = Some((
                            *request,
                            time::after(if words[0] & 0xffff == wire::CONFIGURE {
                                wire::CONFIGURE_DEADLINE_TICKS
                            } else {
                                100
                            })?,
                            Purpose::Owner,
                        ));
                        self.retiring[slot] = None;
                        return Ok(true);
                    }
                    Err(error) => Err(error),
                }
            }
        } else {
            match words {
                [
                    wire::LAUNCH_IMAGES,
                    slot @ 0..=LAST_CHILD,
                    images,
                    0,
                    0,
                    0,
                    0,
                    0,
                ] if images >> abi::MAX_LAUNCHES == 0 => {
                    let result = (0..abi::MAX_LAUNCHES)
                        .filter(|i| images & (1 << i) != 0)
                        .try_for_each(|i| {
                            cathedral_user_runtime::task::Executable::at(i as u64).map(|_| ())
                        });
                    result.map(|()| self.allowed_images[slot as usize] = images)
                }
                [wire::LAUNCHER_FALLBACK, image, argument, 0, 0, 0, 0, 0] => {
                    self.set_fallback(image, argument)
                }
                [
                    wire::MANAGEMENT_GRANT,
                    manager @ 0..=LAST_CHILD,
                    target @ 0..=5,
                    0,
                    0,
                    0,
                    0,
                    0,
                ] => {
                    if target == 0 {
                        self.management.revoke(manager as usize);
                        Ok(())
                    } else {
                        server
                            .incarnation(manager as usize)
                            .ok_or(Error(abi::DENIED as i64))
                            .and_then(|incarnation| {
                                self.management
                                    .grant(manager as usize, incarnation, (target - 1) as usize)
                                    .map_err(|c| Error(c as i64))
                            })
                    }
                }
                [wire::CONFIGURE, 0, 0, 0, 0, 0, 0, 0] => self.configure(parent, world, server),
                [wire::RESTORE_STORAGE, 0, 0, 0, 0, 0, 0, 0] => {
                    self.restore_storage(parent, server)
                }
                [
                    wire::LAUNCH_IMPORTS,
                    slot @ 0..=LAST_CHILD,
                    offered,
                    0,
                    0,
                    0,
                    0,
                    0,
                ] if offered & !cathedral_platform_contracts::imports::SUPPORTED == 0 => {
                    self.allowed_imports[slot as usize] = Some(offered);
                    Ok(())
                }
                [
                    wire::RUN_PAGES,
                    slot @ 0..=LAST_CHILD,
                    private,
                    readers,
                    0,
                    0,
                    0,
                    0,
                ] if private <= cathedral_kernel_contracts::memory::MAX_PRIVATE_PAGES as u64
                    && readers <= cathedral_kernel_contracts::memory::MAX_READER_PAGES as u64 =>
                {
                    if self.slots[slot as usize].is_some() {
                        Err(Error(abi::BUSY as i64))
                    } else {
                        self.next_pages[slot as usize] = Some([private, readers]);
                        Ok(())
                    }
                }
                [
                    wire::RUN_MEMORY,
                    slot @ 0..=LAST_CHILD,
                    frames,
                    0,
                    0,
                    0,
                    0,
                    0,
                ] => {
                    if self.slots[slot as usize].is_some() {
                        Err(Error(abi::BUSY as i64))
                    } else {
                        self.next_frames[slot as usize] = Some(frames);
                        Ok(())
                    }
                }
                [
                    wire::LAUNCH_POLICY,
                    slot @ 0..=LAST_CHILD,
                    private,
                    readers,
                    channels,
                    descendants,
                    cpu,
                    surfaces,
                ] if private <= cathedral_kernel_contracts::memory::MAX_PRIVATE_PAGES as u64
                    && readers <= cathedral_kernel_contracts::memory::MAX_READER_PAGES as u64
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
                    slot @ 0..=LAST_CHILD,
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
                [
                    wire::SPAWN,
                    slot @ 0..=LAST_CHILD,
                    launch,
                    loans,
                    argument,
                    0,
                    0,
                    0,
                ] => self.start(slot as usize, launch, loans, argument, server, world),
                [
                    wire::RUN,
                    slot @ 0..=LAST_CHILD,
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
                [wire::STOP, slot @ 0..=LAST_CHILD, 0, 0, 0, 0, 0, 0] => {
                    if slot == 0 {
                        self.launcher = None;
                    }
                    self.stop(slot as usize, server, world, parent)
                }
                _ => Err(Error(abi::DENIED as i64)),
            }
        };
        if result.is_ok() {
            self.recovery_owner_changed(words);
        }
        server.reply(
            request,
            &wire::encode(wire::error(result.err().map_or(0, |e| e.0 as u64))),
        )?;
        Ok(true)
    }
}
