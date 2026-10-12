// One level of ownership. The same state machine runs above apps or other worlds.
use cathedral_contracts::{matrix as wire, user as abi};
use children::Child;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Grant {
    pub epoch: u64,
    pub width: u64,
    pub height: u64,
    pub keys: u64,
}
impl Grant {
    pub fn valid(self) -> bool {
        self.epoch != 0
            && self.width > 0
            && self.height > 0
            && self.width <= wire::MAX_EXTENT
            && self.height <= wire::MAX_EXTENT
            && self.keys & !wire::KEY_MASK == 0
    }
}
#[derive(Default)]
pub struct World {
    parent: Grant,
    capacity: usize,
    retired: [Option<wire::Packet>; wire::MAX_SURFACES],
    children: [Child; 2],
    focus: Option<usize>,
    capture: Option<usize>,
    buttons: u64,
    blocked: bool,
    held: u64,
    suppressed: u64,
}
impl World {
    /// Retained process state can be installed without a large stack temporary.
    pub const fn empty() -> Self {
        Self {
            parent: Grant {
                epoch: 0,
                width: 0,
                height: 0,
                keys: 0,
            },
            capacity: 0,
            retired: [None; wire::MAX_SURFACES],
            children: [const { Child::empty() }; 2],
            focus: None,
            capture: None,
            buttons: 0,
            blocked: false,
            held: 0,
            suppressed: 0,
        }
    }

    pub fn inherit(&mut self, grant: Grant) {
        let grant = if grant.valid() {
            grant
        } else {
            Grant::default()
        };
        if self.parent == grant {
            return;
        }
        let replaced = self.parent.epoch != grant.epoch;
        if replaced {
            self.cancel();
            if grant.valid() {
                // A newly accepted binding is a fresh scoped stream. Its parent
                // suppresses old physical holds before publishing new events.
                self.buttons = 0;
                self.blocked = false;
                self.held = 0;
                self.suppressed = 0;
            }
        }
        if self.parent.valid() && (replaced || !grant.valid()) {
            // Losing the parent's authority cannot be repaired by silently reviving
            // old descendant grants when a replacement parent later reconnects.
            for slot in 0..2 {
                self.retire_surfaces(slot);
                self.children[slot].rect = None;
                self.children[slot].surface_limit = 0;
            }
        }
        self.parent = grant;
        let mut changed = [false; 2];
        for (slot, child) in self.children.iter_mut().enumerate() {
            let previous = child.grant;
            child.refresh(grant, replaced);
            if child.grant != previous {
                changed[slot] = true;
                if self.focus == Some(slot) {
                    self.focus = None;
                    self.suppressed |= self.held;
                }
                if self.capture == Some(slot) {
                    self.capture = None;
                }
            }
        }
        for (slot, changed) in changed.into_iter().enumerate() {
            if changed {
                self.retire_surfaces(slot);
            }
        }
    }
    pub fn place(&mut self, slot: usize, rect: Rect, keys: u64) -> Result<(), u64> {
        if !self.parent.valid() || !rect.valid() || keys & !self.parent.keys != 0 {
            return Err(abi::DENIED);
        }
        let child = self.children.get_mut(slot).ok_or(abi::DENIED)?;
        let previous = child.grant;
        child.rect = Some(rect);
        child.keys = keys;
        child.refresh(self.parent, false);
        if child.grant != previous {
            self.retire_surfaces(slot);
            if self.focus == Some(slot) {
                self.focus = None;
                self.suppressed |= self.held;
            }
            if self.capture == Some(slot) {
                self.capture = None;
            }
        }
        Ok(())
    }
    pub fn revoke(&mut self, slot: usize) -> Result<(), u64> {
        if slot >= self.children.len() {
            return Err(abi::DENIED);
        }
        if self.focus == Some(slot) {
            self.focus = None;
            self.suppressed |= self.held;
        }
        if self.capture == Some(slot) {
            self.capture = None;
        }
        self.children[slot].rect = None;
        self.retire_surfaces(slot);
        self.children[slot].surface_limit = 0;
        self.children[slot].refresh(self.parent, false);
        Ok(())
    }
    pub fn disconnected(&mut self, slot: usize) {
        let _ = self.revoke(slot);
        if let Some(child) = self.children.get_mut(slot) {
            child.incarnation = 0;
        }
    }
    /// Coalesced wake hint; clients re-read only their local view and events.
    pub fn take_notification(&mut self, slot: usize) -> bool {
        core::mem::take(&mut self.children[slot].notify)
    }
    /// Identity is supplied by the accepted endpoint, never a request field.
    pub fn request(&mut self, slot: usize, incarnation: u64, words: wire::Packet) -> wire::Packet {
        if slot >= self.children.len() {
            return wire::error(abi::DENIED);
        }
        let previous = self.children[slot].incarnation;
        if previous != 0 && previous != incarnation {
            self.disconnected(slot);
        }
        let child = &mut self.children[slot];
        child.incarnation = incarnation;
        match words {
            [wire::VIEW, 0, 0, 0, 0, 0, 0, 0] => child.view(),
            [wire::POLL, 0, 0, 0, 0, 0, 0, 0] if child.grant.valid() => child.poll(),
            [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0] if child.grant.valid() => [
                0,
                child.surface_limit as u64,
                child.surfaces.iter().flatten().count() as u64,
                0,
                0,
                0,
                0,
                0,
            ],
            [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0]
                if child.grant.valid() && epoch == child.grant.epoch =>
            {
                child.next()
            }
            _ => wire::error(abi::DENIED),
        }
    }
}
