//! Control owns placement; client connections own snapshots. No client-supplied owner IDs.
use super::*;
use cathedral_user_runtime::{link::Link, memory::Shared, server::Request};
impl Compositor {
    pub(super) fn request(&mut self, request: &Request) -> [u64; 6] {
        let result = self.dispatch(request);
        result.unwrap_or_else(|status| [status, 0, 0, 0, 0, 0])
    }
    fn dispatch(&mut self, request: &Request) -> Result<[u64; 6], u64> {
        let words = display::decode(&request.bytes[..request.len]).ok_or(abi::INVALID_ARGUMENT)?;
        if !request.control {
            let client = request.client.ok_or(abi::DENIED)?;
            if words[0] != wire::SUBMIT {
                return Err(abi::DENIED);
            }
            // Authenticate and consume this client's offer even when its revision or
            // scene is invalid. A foreign offer remains untouched for its real owner.
            let link = Link::at(client as u64).map_err(|e| e.0 as u64)?;
            let region = Shared::accept_from(words[2], link).map_err(|e| e.0 as u64)?;
            let result = if words[3..].iter().any(|&n| n != 0) {
                Err(abi::INVALID_ARGUMENT)
            } else {
                self.clients[client].submit(
                    request.incarnation,
                    words[1],
                    region.bytes(),
                    self.scratch,
                )
            };
            drop(region); // Completion always follows release of the accepted reader.
            result?;
            self.redraw(self.clients[client].scope);
            return Ok([0, words[1], 0, 0, 0, 0]);
        }
        match words {
            [wire::BACKGROUND, color, 0, 0, 0, 0] if color <= 0xffffff => {
                self.background = color as u32;
                self.redraw(Rect {
                    x: 0,
                    y: 0,
                    w: self.output.width,
                    h: self.output.height,
                });
            }
            [wire::PLACE, client, x, y, w, h] if client < 2 => {
                let rect = Rect { x, y, w, h };
                if !rect.valid()
                    || w > 256
                    || h > 256
                    || x + w > self.output.width
                    || y + h > self.output.height
                {
                    return Err(abi::INVALID_ARGUMENT);
                }
                let old = self.clients[client as usize].scope;
                self.clients[client as usize].scope = rect;
                self.redraw(old);
                self.redraw(rect);
            }
            [wire::PIXEL, x, y, 0, 0, 0] if x < self.output.width && y < self.output.height => {
                return Ok([0, self.output.read(x, y) as u64, 0, 0, 0, 0]);
            }
            [wire::STATUS, client, 0, 0, 0, 0] if client < 2 => {
                let c = &self.clients[client as usize];
                return Ok([0, c.incarnation, c.revision, u64::from(c.visible), 0, 0]);
            }
            _ => return Err(abi::INVALID_ARGUMENT),
        }
        Ok([0; 6])
    }
}
