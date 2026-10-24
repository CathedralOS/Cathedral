// Owned surface names, conserving allowances and one-level clipping/translation.
// Upstream names and stacking offsets never cross a child-facing connection.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Surface {
    name: u64,
    upstream: u64,
    draw: Option<wire::Packet>,
}
impl<const N: usize> World<N> {
    pub fn surface_capacity(&mut self, capacity: usize) {
        let capacity = capacity.min(wire::MAX_SURFACES);
        if capacity == self.capacity {
            return;
        }
        self.authority = self.authority.checked_add(1).expect("authority generation");
        for slot in 0..N {
            self.retire_surfaces(slot);
            self.children[slot].surface_limit = 0;
            self.children[slot].frame.pending = None;
            self.children[slot].demand(self.frames && self.children[slot].visible);
            self.children[slot].notify = true;
        }
        self.capacity = capacity;
    }
    pub fn surface_limit(&mut self, slot: usize, count: usize) -> Result<(), u64> {
        if slot >= N || count > self.capacity {
            return Err(abi::NO_MEMORY);
        }
        if self.children[slot].surface_limit == count {
            return Ok(());
        }
        // Pick a free rank range without shifting any sibling's live ranks.
        let base = (0..=self.capacity - count)
            .find(|base| {
                self.children.iter().enumerate().all(|(i, child)| {
                    i == slot
                        || count == 0
                        || child.surface_limit == 0
                        || base + count <= child.surface_base
                        || *base >= child.surface_base + child.surface_limit
                })
            })
            .ok_or(abi::NO_MEMORY)?;
        self.surface_range(slot, base, count)
    }
    pub fn backed_surfaces(&self) -> usize {
        self.capacity
    }
    /// Owner-selected stacking range. Allocation order need not determine z-order.
    /// Refuse overlap or relocating live surfaces; siblings never move implicitly.
    pub fn surface_range(&mut self, slot: usize, base: usize, count: usize) -> Result<(), u64> {
        if slot >= N
            || base
                .checked_add(count)
                .is_none_or(|end| end > self.capacity)
            || self.children.iter().enumerate().any(|(i, child)| {
                i != slot
                    && count != 0
                    && child.surface_limit != 0
                    && base < child.surface_base + child.surface_limit
                    && base + count > child.surface_base
            })
        {
            return Err(abi::NO_MEMORY);
        }
        let child = &mut self.children[slot];
        if child.surface_limit == count && child.surface_base == base {
            return Ok(());
        }
        if child.surfaces.iter().any(Option::is_some) {
            return Err(abi::BUSY);
        }
        child.surface_base = base;
        child.surface_limit = count;
        child.frame.pending = None;
        child.demand(self.frames && child.visible);
        child.notify = true;
        Ok(())
    }
    /// Validate before upstream allocation. Returns only a provider-internal rank.
    pub fn prepare_surface(
        &mut self,
        slot: usize,
        incarnation: u64,
        epoch: u64,
        rank: u64,
    ) -> Result<u64, u64> {
        let view = self.request(slot, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        if view[0] != 0 || view[1] != epoch {
            return Err(abi::DENIED);
        }
        let child = &self.children[slot];
        if child.surfaces.iter().flatten().count() == child.surface_limit {
            return Err(abi::NO_MEMORY);
        }
        if rank >= child.surface_limit as u64 {
            return Err(abi::DENIED);
        }
        if child.surfaces[rank as usize].is_some() {
            return Err(abi::BUSY);
        }
        child.next_surface.checked_add(1).ok_or(abi::NO_MEMORY)?;
        let offset = child.surface_base;
        Ok(rank + offset as u64)
    }
    /// Sole service loop commits after upstream success; no dispatch occurs in between.
    pub fn create_surface(&mut self, slot: usize, rank: usize, upstream: u64) -> u64 {
        let child = &mut self.children[slot];
        assert!(rank < child.surface_limit && child.surfaces[rank].is_none());
        child.next_surface = child.next_surface.checked_add(1).unwrap();
        let name = child.next_surface;
        child.surfaces[rank] = Some(Surface {
            name,
            upstream,
            draw: None,
        });
        name
    }
    pub fn surface(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
    ) -> Result<wire::Packet, u64> {
        self.update_surface(slot, incarnation, words, true)
    }
    pub fn preview_surface(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
    ) -> Result<wire::Packet, u64> {
        self.update_surface(slot, incarnation, words, false)
    }
    fn update_surface(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
        commit: bool,
    ) -> Result<wire::Packet, u64> {
        let view = self.request(slot, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        if view[0] != 0 || view[1] != words[1] {
            return Err(abi::DENIED);
        }
        let child = &mut self.children[slot];
        let rank = child
            .surfaces
            .iter()
            .position(|s| s.is_some_and(|s| s.name == words[2]))
            .ok_or(abi::BAD_HANDLE)?;
        let surface = child.surfaces[rank].as_mut().unwrap();
        let upstream = surface.upstream;
        if words[0] == wire::ERASE && words[3..] == [0; 5] {
            if commit {
                child.surfaces[rank] = None;
            }
            return Ok([wire::ERASE, self.parent.epoch, upstream, 0, 0, 0, 0, 0]);
        }
        let [op, _, _, x, y, w, h, color] = words;
        if !matches!(op, wire::PAINT | wire::SURFACE_IMAGE | wire::SURFACE_CURSOR)
            || (op != wire::SURFACE_IMAGE && color > 0xffffff)
            || (op == wire::SURFACE_CURSOR
                && (w > cathedral_platform_contracts::display::composition::CURSOR_WIDTH
                    || h > cathedral_platform_contracts::display::composition::CURSOR_HEIGHT))
            || (op == wire::SURFACE_IMAGE
                && (color == 0 || w > color || color > wire::MAX_EXTENT || h > wire::MAX_EXTENT))
            || x.checked_add(w).is_none()
            || y.checked_add(h).is_none()
        {
            return Err(abi::DENIED);
        }
        if commit {
            surface.draw = Some(words);
        }
        let placement = child.rect.unwrap();
        let w = w.min(view[2].saturating_sub(x));
        let h = h.min(view[3].saturating_sub(y));
        if !child.visible || w == 0 || h == 0 {
            return Ok([op, self.parent.epoch, upstream, 0, 0, 0, 0, color]);
        }
        Ok([
            op,
            self.parent.epoch,
            upstream,
            placement.x + x,
            placement.y + y,
            w,
            h,
            color,
        ])
    }
    /// Owner-only capture snapshot in source-local coordinates, independent of
    /// display visibility. The returned backing key stays within the provider.
    /// Caller pins readers before dispatching any later publication.
    pub fn capture_surface(
        &self,
        slot: usize,
        incarnation: u64,
        epoch: u64,
        rank: usize,
    ) -> Result<Option<wire::Packet>, u64> {
        if !self.binding_matches(slot, incarnation, epoch) {
            return Err(abi::DENIED);
        }
        let child = &self.children[slot];
        let Some(surface) = child.surfaces.get(rank).ok_or(abi::DENIED)? else {
            return Ok(None);
        };
        let Some(mut words) = surface.draw else {
            return Ok(None);
        };
        words[1] = epoch;
        words[2] = surface.upstream;
        words[5] = words[5].min(child.grant.width.saturating_sub(words[3]));
        words[6] = words[6].min(child.grant.height.saturating_sub(words[4]));
        Ok((words[5] != 0 && words[6] != 0).then_some(words))
    }
    /// Replay one object at a time: no full-scene temporaries on the user stack.
    pub fn replay(&mut self, slot: usize, rank: usize) -> Option<wire::Packet> {
        let child = &self.children[slot];
        let mut words = child.surfaces[rank]?.draw?;
        words[1] = child.grant.epoch;
        self.surface(slot, child.incarnation, words).ok()
    }
    pub(super) fn retire_surfaces(&mut self, slot: usize) {
        for surface in &mut self.children[slot].surfaces {
            if let Some(surface) = surface.take() {
                *self
                    .retired
                    .iter_mut()
                    .find(|r| r.is_none())
                    .expect("flush between dispatches") = Some([
                    wire::ERASE,
                    self.parent.epoch,
                    surface.upstream,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]);
            }
        }
    }
    pub fn take_retired_surface(&mut self) -> Option<wire::Packet> {
        self.retired
            .iter_mut()
            .find(|r| r.is_some())
            .and_then(Option::take)
    }
}
