// Own direct children and their local connections. Pending owner RPCs leave the
// service available for the child's upstream requests, avoiding recursive deadlock.
// A child may ask for a sibling: the host reads the program out of the child's
// realm and admits it into the slot the owner opened with a standing policy,
// so a launcher is a broker that never sits in what it launched.
// This entrance owns the records. control/owner.rs executes Omega's owner choices;
// launch/request.rs supplies authenticated launch facts and retains the selected grants.
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
    control: control::Control,
    data: Pair,
    storage: Option<Pair>,
    producer: Option<Pair>,
    // None: legacy connection. Some(false): offered but not activated.
    // Some(true): activated once, even if subsequently closed/revoked. Preserve
    // this state so interaction cannot become a fallback production endpoint.
    producer_started: Option<bool>,
    notify: cathedral_user_runtime::services::WakeSource,
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
    desktop: bool,
    requested_surfaces: [Option<u64>; CHILDREN],
    login: bool,
    entered: bool,
    startup_recovery: startup::Recovery,
    catalogue: cathedral_stock_matrix::policy::catalogue::Catalogue,
    selecting: Option<workspace::selection::Pending>,
    slots: [Option<Owned>; CHILDREN],
    saved_storage: [Option<(usize, u64)>; CHILDREN],
    next_frames: [Option<u64>; CHILDREN],
    next_pages: [Option<[u64; 2]>; CHILDREN],
    standing: [Option<Standing>; CHILDREN],
    allowed_imports: [Option<u64>; CHILDREN],
    allowed_images: [u64; CHILDREN],
    pending: [Option<(Request, u64, Purpose)>; CHILDREN],
    retiring: [Option<i64>; CHILDREN],
    close_paused: [bool; CHILDREN],
    /// Test-only: a close whose LOSS waits for the storage hold to end, so the
    /// sibling's clean answer lands after the busy window (delay_probe).
    #[cfg(feature = "interaction-delay")]
    held_loss: [bool; CHILDREN],
    loss: Option<loss::Survey>,
    launcher: Option<recovery::Launcher>,
    launching: Option<launch::Pending>,
    #[cfg(any(feature = "launch-bind-failure", feature = "launch-place-failure"))]
    launch_probe_used: bool,
    operation_tickets: cathedral_stock_matrix::policy::operation::Tickets,
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
                match child.notify.signal() {
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
}
