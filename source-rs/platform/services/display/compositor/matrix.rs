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
        } else if matches!(op & 0xffff, wire::MATRIX_RECT | wire::MATRIX_CURSOR)
            && (op >> 16) < wire::MATRIX_TILES as u64
            && color <= 0xffffff
            && (op & 0xffff != wire::MATRIX_CURSOR
                || (w <= wire::CURSOR_WIDTH && h <= wire::CURSOR_HEIGHT))
        {
            let rect = if w == 0 && h == 0 {
                Rect::default()
            } else if display::valid_rect(self.output.width, self.output.height, x, y, w, h) {
                Rect { x, y, w, h }
            } else {
                return Err(abi::INVALID_ARGUMENT);
            };
            self.publish_or_stage(presentation::Update {
                index: (op >> 16) as usize,
                rect,
                value: color,
                cursor: op & 0xffff == wire::MATRIX_CURSOR,
                object: None,
            })?;
        } else {
            return Err(abi::DENIED);
        }
        Ok([0; 6])
    }
}

impl Compositor {
    pub(super) fn matrix_image(
        &mut self,
        request: &cathedral_user_runtime::server::Request,
        object: cathedral_user_runtime::memory::Object,
    ) -> Result<[u64; 6], u64> {
        let result = self.prepare_image(request, object);
        if request.control
            && let Err(status) = result
        {
            self.poison_presentation(status);
        }
        result
    }
    fn prepare_image(
        &mut self,
        request: &cathedral_user_runtime::server::Request,
        mut object: cathedral_user_runtime::memory::Object,
    ) -> Result<[u64; 6], u64> {
        if !request.control {
            return Err(abi::DENIED);
        }
        let [op, x, y, w, h, stride] =
            display::decode(&request.bytes[..request.len]).ok_or(abi::INVALID_ARGUMENT)?;
        if op & 0xffff != wire::MATRIX_IMAGE
            || op >> 16 >= wire::MATRIX_TILES as u64
            || !cathedral_platform_contracts::display::composition::valid_image(
                w,
                h,
                stride,
                object.len(),
            )
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        let rect = if w == 0 && h == 0 {
            Rect::default()
        } else if display::valid_rect(self.output.width, self.output.height, x, y, w, h) {
            Rect { x, y, w, h }
        } else {
            return Err(abi::INVALID_ARGUMENT);
        };
        object.map().map_err(|e| e.0 as u64)?;
        let index = (op >> 16) as usize;
        self.publish_or_stage(presentation::Update {
            index,
            rect,
            value: stride,
            cursor: false,
            object: Some(object),
        })?;
        Ok([0; 6])
    }
}
