//! Validate a child-local primitive, clip and translate exactly one ownership level.
use super::*;
impl World {
    pub fn surface(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
    ) -> Result<wire::Packet, u64> {
        let view = self.request(slot, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        if view[0] != 0 || view[1] != words[1] || words[2] > 2 {
            return Err(abi::DENIED);
        }
        let key = (words[2] << 2) | (slot as u64 + 1);
        if words[0] == wire::ERASE && words[3..] == [0; 5] {
            return Ok([wire::ERASE, self.parent.epoch, key, 0, 0, 0, 0, 0]);
        }
        let [op, _, _, x, y, w, h, color] = words;
        if op != wire::PAINT
            || color > 0xffffff
            || x.checked_add(w).is_none()
            || y.checked_add(h).is_none()
        {
            return Err(abi::DENIED);
        }
        let placement = self.children[slot].rect.unwrap();
        let w = w.min(view[2].saturating_sub(x));
        let h = h.min(view[3].saturating_sub(y));
        if w == 0 || h == 0 {
            return Ok([wire::PAINT, self.parent.epoch, key, 0, 0, 0, 0, color]);
        }
        Ok([
            wire::PAINT,
            self.parent.epoch,
            key,
            placement.x + x,
            placement.y + y,
            w,
            h,
            color,
        ])
    }
}
