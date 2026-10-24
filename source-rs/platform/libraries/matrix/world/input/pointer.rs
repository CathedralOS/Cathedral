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
        if !self.parent.valid() || buttons > 7 {
            return Err(abi::DENIED);
        }
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
        if route.buttons == 0 && buttons != 0 {
            if route.focus != hit {
                if let Some(old) = route.focus {
                    self.children[old].cancel_stream(index, channel);
                }
                route.suppressed |= route.held;
                route.focus = hit;
            }
            route.capture = hit;
        }
        let target = if route.buttons != 0 {
            route.capture
        } else {
            hit
        };
        route.buttons = buttons;
        if let Some(slot) = target {
            let child = &mut self.children[slot];
            if child.grant.valid() {
                let r = child.rect.unwrap();
                let x = x.saturating_sub(r.x).min(child.grant.width - 1);
                let y = y.saturating_sub(r.y).min(child.grant.height - 1);
                if !child.push(index, channel, wire::POINTER, x, y, buttons) {
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
