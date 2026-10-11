//! Control owns placement; client connections own snapshots. No client-supplied owner IDs.
use super::*;
use cathedral_user_runtime::{link::Link, memory::Shared, server::Request};
impl Compositor {
    pub(super) fn request(&mut self, request: &Request) -> [u64; 6] {
        let result = self.dispatch(request);
        result.unwrap_or_else(|status| [status, 0, 0, 0, 0, 0])
    }
    fn dispatch(&mut self, request: &Request) -> Result<[u64; 6], u64> {
        if request.len == display::TEXT_BYTES {
            if !request.control {
                return Err(abi::DENIED);
            }
            let damage = self.composition.labels.set(
                &request.bytes,
                self.output.width,
                self.output.height,
            )?;
            self.redraw(damage);
            return Ok([0; 6]);
        }
        let words = display::decode(&request.bytes[..request.len]).ok_or(abi::INVALID_ARGUMENT)?;
        if matches!(words[0] & 0xffff, wire::MATRIX_RECT | wire::MATRIX_POINTER) {
            if !request.control {
                return Err(abi::DENIED);
            }
            return self.matrix_request(words);
        }
        if words[0] >= cathedral_contracts::interaction::FOCUS_GRANT {
            return self.session_request(request, words);
        }
        if words[0] >= cathedral_contracts::pointer::DEVICE {
            return self.pointer_request(request, words);
        }
        if !request.control {
            let client = request.client.and_then(app_index).ok_or(abi::DENIED)?;
            if words[0] != wire::SUBMIT {
                return Err(abi::DENIED);
            }
            // Authenticate and consume this client's offer even when its revision or
            // scene is invalid. A foreign offer remains untouched for its real owner.
            let link = Link::at(request.client.unwrap() as u64).map_err(|e| e.0 as u64)?;
            let region = Shared::accept_from(words[2], link).map_err(|e| e.0 as u64)?;
            if self.composition.clients[client].incarnation != request.incarnation {
                self.composition
                    .pointers
                    .client_lost(client, self.composition.clients);
            }
            let was_visible = self.composition.clients[client].visible
                && self.composition.clients[client].incarnation == request.incarnation;
            let result = if words[3..].iter().any(|&n| n != 0) {
                Err(abi::INVALID_ARGUMENT)
            } else {
                self.composition.clients[client].submit(
                    request.incarnation,
                    words[1],
                    region.bytes(),
                    self.composition.scratch,
                )
            };
            drop(region); // Completion always follows release of the accepted reader.
            result?;
            self.composition.tree.resolve(self.composition.clients);
            let c = &self.composition.clients[client];
            let local = c.scene.damage(self.composition.scratch);
            let damage = if was_visible {
                Rect {
                    x: c.scope.x + local.x,
                    y: c.scope.y + local.y,
                    w: local.w,
                    h: local.h,
                }
                .intersect(c.scope)
            } else {
                c.scope
            };
            self.redraw(damage);
            return Ok([0, words[1], 0, 0, 0, 0]);
        }
        if words == [display::INFO, 0, 0, 0, 0, 0] {
            return Ok([0, self.output.width, self.output.height, 0, 0, 0]);
        }
        match words {
            [wire::BACKGROUND, color, 0, 0, 0, 0] if color <= 0xffffff => {
                self.composition.background = color as u32;
                self.redraw(Rect {
                    x: 0,
                    y: 0,
                    w: self.output.width,
                    h: self.output.height,
                });
            }
            [wire::PLACE, client, x, y, w, h] if client < 3 => {
                let rect = Rect { x, y, w, h };
                if !rect.valid()
                    || w > 256
                    || h > 256
                    || x + w > self.output.width
                    || y + h > self.output.height
                {
                    return Err(abi::INVALID_ARGUMENT);
                }
                self.composition.tree.place_root(client as usize, rect)?;
                self.layout_changed();
            }
            [wire::PIXEL, x, y, 0, 0, 0] if x < self.output.width && y < self.output.height => {
                return Ok([0, self.output.read(x, y) as u64, 0, 0, 0, 0]);
            }
            [wire::STATUS, client, 0, 0, 0, 0] if client < 3 => {
                let c = &self.composition.clients[client as usize];
                return Ok([0, c.incarnation, c.revision, u64::from(c.visible), 0, 0]);
            }
            _ => return Err(abi::INVALID_ARGUMENT),
        }
        Ok([0; 6])
    }
}
