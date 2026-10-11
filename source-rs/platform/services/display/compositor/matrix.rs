//! Root-adapter commands share the retained compositor and framebuffer writer.
use super::*;
impl Compositor {
    pub(super) fn matrix_request(&mut self, words: [u64; 6]) -> Result<[u64; 6], u64> {
        let [op, x, y, w, h, color] = words;
        if op == wire::MATRIX_POINTER
            && [w, h, color] == [0; 3]
            && x < self.output.width
            && y < self.output.height
        {
            let before = self.composition.pointers.cursor_bounds();
            let state = self.composition.pointers.state(0);
            self.composition.pointers.device(
                x as i64 - state[1] as i64,
                y as i64 - state[2] as i64,
                0,
                false,
                self.composition.clients,
            );
            self.redraw_cursors(before);
        } else if op & 0xffff == wire::MATRIX_RECT
            && wire::MATRIX_KEYS.contains(&(op >> 16))
            && color <= 0xffffff
        {
            let rect = if w == 0 && h == 0 {
                Rect::default()
            } else if display::valid_rect(self.output.width, self.output.height, x, y, w, h) {
                Rect { x, y, w, h }
            } else {
                return Err(abi::INVALID_ARGUMENT);
            };
            let damage = self
                .composition
                .tiles
                .set((op >> 16) as usize, rect, color as u32);
            self.redraw(damage);
        } else {
            return Err(abi::DENIED);
        }
        Ok([0; 6])
    }
}
