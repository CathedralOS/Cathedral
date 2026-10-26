//! Early configuration and the independent restart screen. These operations
//! are shared by the presentation owner and the separately supplied boot port;
//! bootstrap has no scene publication, input injection or client-grant authority.
use super::*;
use cathedral_platform_contracts::display::pointer;
use cathedral_user_runtime::server::Request;
impl Compositor {
    pub(super) fn configuration(&mut self, request: &Request) -> Option<Result<[u64; 6], u64>> {
        if request.len == display::TEXT_BYTES {
            return Some(self.label(request));
        }
        let words = display::decode(&request.bytes[..request.len])?;
        Some(match words {
            [display::INFO, 0, 0, 0, 0, 0] => {
                Ok([0, self.output.width, self.output.height, 0, 0, 0])
            }
            [wire::BACKGROUND, color, 0, 0, 0, 0] if color <= 0xffffff => {
                self.composition.background = color as u32;
                self.redraw(Rect {
                    x: 0,
                    y: 0,
                    w: self.output.width,
                    h: self.output.height,
                });
                Ok([0; 6])
            }
            [pointer::ENABLE, enabled @ 0..=1, 0, 0, 0, 0] => {
                let before = self.composition.pointers.cursor_bounds();
                self.composition
                    .pointers
                    .enable(enabled == 1, self.composition.clients);
                self.redraw_cursors(before);
                Ok([0; 6])
            }
            [pointer::ENABLE, ..] => Err(abi::DENIED),
            [display::INFO | wire::BACKGROUND, ..] => Err(abi::INVALID_ARGUMENT),
            _ => return None,
        })
    }
    fn label(&mut self, request: &Request) -> Result<[u64; 6], u64> {
        let damage =
            self.composition
                .labels
                .set(&request.bytes, self.output.width, self.output.height)?;
        self.redraw(damage);
        Ok([0; 6])
    }
}
