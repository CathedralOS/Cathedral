//! Per-stream hit testing, selection and drag capture.
use super::*;
impl<const N: usize> World<N> {
    pub fn pointer_stream(
        &mut self,
        channel: u32,
        x: u64,
        y: u64,
        buttons: u64,
    ) -> Result<(), u64> {
        self.pointer_named(Name::local(channel), x, y, buttons)
    }
    pub(in crate::world) fn pointer_named(
        &mut self,
        channel: Name,
        x: u64,
        y: u64,
        buttons: u64,
    ) -> Result<(), u64> {
        self.pointer_routed(channel, x, y, buttons, true)
    }
    pub(super) fn pointer_routed(
        &mut self,
        channel: Name,
        x: u64,
        y: u64,
        buttons: u64,
        select: bool,
    ) -> Result<(), u64> {
        if !self.parent.valid() || buttons > 7 {
            return Err(abi::DENIED);
        }
        self.reap_input();
        let index = self.input.admit(channel)?;
        let route = &mut self.input.streams[index];
        if route.blocked {
            if buttons == 0 {
                route.blocked = false;
            }
            return Ok(());
        }
        let x = x.min(self.parent.width - 1);
        let y = y.min(self.parent.height - 1);
        let hit = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.visible
                    && c.grant.valid()
                    && c.rect.is_some_and(|r| {
                        x >= r.x && y >= r.y && x - r.x < c.grant.width && y - r.y < c.grant.height
                    })
            })
            // Follow the owner's surface stacking; input-only equal ranges use slot order.
            .max_by_key(|(slot, c)| (c.surface_base, *slot))
            .map(|(slot, _)| slot);
        let target = if route.buttons != 0 {
            route.capture
        } else {
            hit
        };
        if let Some(slot) = target {
            self.children[slot].can_push(index, route.group, self.input.groups[route.group].id)?;
        }
        if route.buttons == 0 && buttons != 0 {
            let group = route.group;
            if select {
                self.select_input_group_index(group, hit);
            }
            self.input.streams[index].capture = hit;
        }
        let route = &mut self.input.streams[index];
        route.buttons = buttons;
        if let Some(slot) = target {
            let child = &mut self.children[slot];
            if child.grant.valid() {
                let r = child.rect.unwrap();
                let x = x.saturating_sub(r.x).min(child.grant.width - 1);
                let y = y.saturating_sub(r.y).min(child.grant.height - 1);
                if !child.push(
                    index,
                    channel,
                    route.group,
                    self.input.groups[route.group].id,
                    [wire::POINTER, x, y, buttons],
                )? {
                    // This source may hold keys in a different child from the
                    // hovered pointer target. Interrupt every recipient.
                    self.cancel_stream_index(index);
                    return Ok(());
                }
            }
        }
        if buttons == 0 {
            route.capture = None;
        }
        Ok(())
    }
}
