//! Render demand composes through owners independently of input or execution.
//! One pending owner-requested frame adds independent demand until the owner
//! releases it. Hints do not enforce CPU use or publishing.
use super::*;
impl<const N: usize> World<N> {
    pub fn render_demand(&mut self, frames: bool) {
        if self.frames == frames {
            return;
        }
        self.frames = frames;
        for child in &mut self.children {
            child.demand(frames && child.visible);
        }
    }
    /// Optional owner policy, independent of hiding pixels. Retains the child's
    /// view and input grant but cancels current holds and forgets local routes.
    pub fn deactivate_input(&mut self, slot: usize) -> Result<(), u64> {
        let child = self.children.get_mut(slot).ok_or(abi::DENIED)?;
        child.cancel();
        self.input.forget(slot);
        Ok(())
    }
}

/// Provider-owned request bookkeeping. Serial survives view and connection loss.
pub(super) struct Frame {
    serial: u64,
    pub pending: Option<(u64, bool)>,
}
impl Frame {
    pub const fn empty() -> Self {
        Self {
            serial: 0,
            pending: None,
        }
    }
}
impl<const N: usize> World<N> {
    /// Owner-only operation after capture admission/funding. One request per
    /// source; a provider can coalesce consumers outside this mechanism.
    pub fn request_frame(&mut self, slot: usize, incarnation: u64, epoch: u64) -> Result<u64, u64> {
        if !self.binding_matches(slot, incarnation, epoch) {
            return Err(abi::DENIED);
        }
        let child = &mut self.children[slot];
        if child.frame.pending.is_some() {
            return Err(abi::BUSY);
        }
        let ticket = child.frame.serial.checked_add(1).ok_or(abi::NO_MEMORY)?;
        child.frame.serial = ticket;
        child.frame.pending = Some((ticket, false));
        child.demand(self.frames && child.visible);
        child.notify = true;
        Ok(ticket)
    }
    pub fn frame_ready(&self, slot: usize, incarnation: u64, epoch: u64, ticket: u64) -> bool {
        self.binding_matches(slot, incarnation, epoch)
            && self.children[slot].frame.pending == Some((ticket, true))
    }
    /// Release after capture pins its source/result, timeout, refusal or consumer
    /// loss. Stale cleanup must not remove a replacement request's demand.
    pub fn release_frame(
        &mut self,
        slot: usize,
        incarnation: u64,
        epoch: u64,
        ticket: u64,
    ) -> Result<(), u64> {
        if !self.binding_matches(slot, incarnation, epoch)
            || self.children[slot]
                .frame
                .pending
                .is_none_or(|(t, _)| t != ticket)
        {
            return Err(abi::DENIED);
        }
        let child = &mut self.children[slot];
        child.frame.pending = None;
        child.demand(self.frames && child.visible);
        Ok(())
    }
}
