//! An explicit controller grant permits geometry changes inside this world only.
//! All geometry is validated before any child changes. Existing key masks,
//! revoked views, surface allowances and process identities remain authoritative.
use super::*;

/// Owner-held placement snapshot, never a child-visible capability. Only the
/// owner holding this snapshot may reissue it to a replacement task.
#[derive(Clone, Copy)]
pub struct Placement {
    slot: usize,
    parent: Grant,
    authority: u64,
    rect: Rect,
    keys: u64,
    surfaces: usize,
    surface_base: usize,
}

impl<const N: usize> World<N> {
    pub fn placement(&self, slot: usize, incarnation: u64) -> Option<Placement> {
        let child = self.children.get(slot)?;
        if self.layout_controller != Some((slot, incarnation))
            || !child.grant.valid()
            || child.surface_limit == 0
        {
            return None;
        }
        Some(Placement {
            slot,
            parent: self.parent,
            authority: self.authority,
            rect: child.rect?,
            keys: child.keys,
            surfaces: child.surface_limit,
            surface_base: child.surface_base,
        })
    }
    pub fn parent_matches(&self, placement: Placement) -> bool {
        self.parent == placement.parent
            && self.parent.valid()
            && self.authority == placement.authority
    }
    /// Reissue only the captured view. Sibling scenes, epochs, focus, input
    /// and surface allowance are untouched; a changed ancestor grant refuses.
    pub fn restore(
        &mut self,
        placement: Placement,
        incarnation: u64,
        controls_layout: bool,
    ) -> Result<(), u64> {
        if !self.parent_matches(placement)
            || incarnation == 0
            || placement.slot >= N
            || self.children[placement.slot].rect.is_some()
            || self.layout_controller.is_some()
        {
            return Err(abi::DENIED);
        }
        self.surface_range(placement.slot, placement.surface_base, placement.surfaces)?;
        self.place(placement.slot, placement.rect, placement.keys)?;
        if controls_layout {
            self.grant_layout(placement.slot, incarnation);
        }
        Ok(())
    }

    /// The owner chooses the controller; a child cannot grant this to itself.
    pub fn grant_layout(&mut self, slot: usize, incarnation: u64) {
        assert!(slot < N && incarnation != 0);
        self.layout_controller = Some((slot, incarnation));
    }

    pub fn layout(
        &mut self,
        slot: usize,
        incarnation: u64,
        epoch: u64,
        rects: [Rect; N],
    ) -> Result<[Rect; N], u64> {
        if !self.layout_allowed(slot, incarnation, epoch) {
            return Err(abi::DENIED);
        }
        if rects.iter().any(|r| {
            !r.valid()
                || r.x
                    .checked_add(r.width)
                    .is_none_or(|end| end > self.parent.width)
                || r.y
                    .checked_add(r.height)
                    .is_none_or(|end| end > self.parent.height)
        }) {
            return Err(abi::INVALID_ARGUMENT);
        }
        for (child, rect) in rects.iter().enumerate() {
            // Revocation is not undone by moving the remaining live views.
            if self.children[child].rect.is_some() {
                let keys = self.children[child].keys & self.parent.keys;
                self.place(child, *rect, keys).expect("validated layout");
            }
        }
        Ok(rects)
    }
    pub fn layout_allowed(&self, slot: usize, incarnation: u64, epoch: u64) -> bool {
        self.layout_controller == Some((slot, incarnation))
            && self.binding_matches(slot, incarnation, epoch)
    }
}
