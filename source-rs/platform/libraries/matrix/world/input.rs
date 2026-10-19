//! Route within this world, clamp capture, cancel focus and attenuate physical keys.
use super::*;
use cathedral_platform_contracts::input;
impl<const N: usize> World<N> {
    pub fn cancel(&mut self) {
        for child in &mut self.children {
            child.cancel();
        }
        self.focus = None;
        self.capture = None;
        self.suppressed |= self.held;
        self.blocked |= self.buttons != 0;
        self.buttons = 0;
    }
    pub fn pointer(&mut self, x: u64, y: u64, buttons: u64) {
        if !self.parent.valid() || buttons > 7 {
            return;
        }
        if self.blocked {
            if buttons == 0 {
                self.blocked = false;
            }
            return;
        }
        let x = x.min(self.parent.width - 1);
        let y = y.min(self.parent.height - 1);
        let hit = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.grant.valid()
                    && c.rect.is_some_and(|r| {
                        x >= r.x && y >= r.y && x - r.x < c.grant.width && y - r.y < c.grant.height
                    })
            })
            // Follow the owner's surface stacking; input-only equal ranges use slot order.
            .max_by_key(|(slot, c)| (c.surface_base, *slot))
            .map(|(slot, _)| slot);
        if self.buttons == 0 && buttons != 0 {
            if self.focus != hit {
                if let Some(old) = self.focus {
                    self.children[old].cancel();
                }
                self.suppressed |= self.held;
                self.focus = hit;
            }
            self.capture = hit;
        }
        let target = if self.buttons != 0 { self.capture } else { hit };
        self.buttons = buttons;
        if let Some(slot) = target {
            let child = &mut self.children[slot];
            if child.grant.valid() {
                let r = child.rect.unwrap();
                let x = x.saturating_sub(r.x).min(child.grant.width - 1);
                let y = y.saturating_sub(r.y).min(child.grant.height - 1);
                if !child.push(wire::POINTER, x, y, buttons) {
                    self.capture = None;
                    if self.focus == Some(slot) {
                        self.suppressed |= self.held;
                    }
                }
            }
        }
        if buttons == 0 {
            self.capture = None;
        }
    }
    pub fn key(&mut self, key: u64, state: u64) {
        if !(1..=input::LAST_KEY as u64).contains(&key)
            || key == input::IDLE as u64
            || ![
                input::PRESS as u64,
                input::RELEASE as u64,
                input::REPEAT as u64,
            ]
            .contains(&state)
        {
            return;
        }
        let bit = 1 << (key - 1);
        if state != input::PRESS as u64 && self.held & bit == 0 {
            return;
        }
        if state == input::RELEASE as u64 {
            self.held &= !bit;
            if self.suppressed & bit != 0 {
                self.suppressed &= !bit;
                return;
            }
        } else {
            self.held |= bit;
            if self.suppressed & bit != 0 {
                return;
            }
        }
        if let Some(slot) = self.focus {
            let child = &mut self.children[slot];
            if child.grant.keys & bit != 0 && !child.push(wire::KEY, key, state, 0) {
                self.suppressed |= self.held;
            }
        }
    }
}
