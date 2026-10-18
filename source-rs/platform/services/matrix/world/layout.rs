//! An explicit launcher grant permits geometry changes inside this world only.
//! All geometry is validated before either child changes. Existing key masks,
//! revoked views, surface allowances and process identities remain authoritative.
use super::*;

impl World {
    /// Called by successful stock configuration, never by a child wire request.
    pub fn grant_layout(&mut self, slot: usize, incarnation: u64) {
        assert!(slot < 2 && incarnation != 0);
        self.layout_controller = Some((slot, incarnation));
    }

    pub fn layout(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
    ) -> Result<[Rect; 2], u64> {
        if !self.layout_allowed(slot, incarnation, words[1]) {
            return Err(abi::DENIED);
        }
        let [wire::APPLY_LAYOUT, _, margin, strip, gap, 0, 0, 0] = words else {
            return Err(abi::INVALID_ARGUMENT);
        };
        let rects =
            wire::config::rectangles(margin, strip, gap, self.parent.width, self.parent.height)
                .ok_or(abi::INVALID_ARGUMENT)?
                .map(|[x, y, width, height]| Rect {
                    x,
                    y,
                    width,
                    height,
                });
        if rects.iter().any(|r| !r.valid()) {
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
