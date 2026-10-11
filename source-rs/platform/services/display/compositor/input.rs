//! Connection-authorized pointer ingress, client delivery and cursor damage.
use super::*;
use cathedral_contracts::pointer as p;
use cathedral_user_runtime::server::Request;
impl Compositor {
    pub(super) fn pointer_request(&mut self, r: &Request, w: [u64; 6]) -> Result<[u64; 6], u64> {
        let before = self.pointers.cursor_bounds();
        let result = if r.control {
            match w {
                [p::ENABLE, enabled @ 0..=1, 0, 0, 0, 0] => {
                    self.pointers.enable(enabled == 1, self.clients);
                    Ok([0; 6])
                }
                [p::LEASE, client @ 0..=2, grant @ 0..=1, 0, 0, 0] => self
                    .pointers
                    .lease(client as usize, grant == 1, self.clients)
                    .map(|epoch| [0, epoch, 0, 0, 0, 0]),
                [p::WAKE, client @ 0..=2, 0, 0, 0, 0] => {
                    self.wake[client as usize] = true;
                    Ok([0; 6])
                }
                [p::STATS, 0, 0, 0, 0, 0] => Ok([
                    0,
                    self.stats[0],
                    self.stats[1],
                    self.stats[2],
                    self.stats[3],
                    0,
                ]),
                [p::STATE, id @ 0..=1, 0, 0, 0, 0] => Ok(self.pointers.state(id as usize)),
                _ => Err(abi::DENIED),
            }
        } else if r.client == Some(2) {
            match w {
                [p::DEVICE, dx, dy, buttons @ 0..=7, reset @ 0..=1, 0] => {
                    if self.source != r.incarnation {
                        self.pointers.source_lost(self.clients);
                        self.source = r.incarnation;
                    }
                    let previous = self.pointers.state(0)[3];
                    self.pointers.device(
                        dx as i64,
                        dy as i64,
                        buttons as u8,
                        reset == 1,
                        self.clients,
                    );
                    if reset == 1 {
                        self.focus.source_lost();
                    } else if previous == 0 && self.pointers.state(0)[3] != 0 {
                        let state = self.pointers.state(0);
                        let target = self.clients.iter().rposition(|c| c.hit(state[1], state[2]));
                        self.focus.click(target, &self.tree, self.clients);
                    }
                    Ok([0; 6])
                }
                _ => Err(abi::DENIED),
            }
        } else {
            let client = r
                .client
                .and_then(app_index)
                .filter(|&c| {
                    c < 3 && self.clients[c].visible && self.clients[c].incarnation == r.incarnation
                })
                .ok_or(abi::DENIED)?;
            match w {
                [p::NEXT, 0, 0, 0, 0, 0] => Ok(self.pointers.next(client).words()),
                [p::CAPTURE, id @ 0..=1, take @ 0..=1, 0, 0, 0] => self
                    .pointers
                    .capture(client, id as usize, take == 1)
                    .map(|()| [0; 6]),
                [p::DRIVE, epoch, x, y, buttons @ 0..=7, 0] => self
                    .pointers
                    .drive(client, epoch, x, y, buttons as u8, self.clients)
                    .map(|()| [0; 6]),
                _ => Err(abi::DENIED),
            }
        };
        // Polling/capture/state do not change cursor pixels.
        if matches!(w[0], p::DEVICE | p::DRIVE | p::LEASE | p::ENABLE) {
            self.redraw_cursors(before);
        }
        result
    }
    pub(super) fn redraw_cursors(&mut self, before: [Rect; 2]) {
        for rect in before.into_iter().chain(self.pointers.cursor_bounds()) {
            self.redraw(rect);
        }
    }
    pub(super) fn disconnected(&mut self, client: usize, incarnation: u64) {
        let before = self.pointers.cursor_bounds();
        if client == 2 {
            if self.source == incarnation {
                self.pointers.source_lost(self.clients);
                self.focus.source_lost();
                self.source = 0;
                cathedral_user_runtime::write(
                    b"Cathedral pointer: source disconnected; capture cancelled\n",
                )
                .unwrap();
            }
        } else if let Some(client) = app_index(client)
            && self.clients[client].incarnation == incarnation
        {
            self.pending[client] = None;
            self.focus.revoke_branch(client, &self.tree);
            self.tree.lost(client);
            for descendant in 0..3 {
                if self.tree.within(client, descendant) {
                    self.pointers.scope_lost(descendant, self.clients);
                }
            }
            self.pointers.client_lost(client, self.clients);
            self.clients[client].disconnect(incarnation);
            self.layout_changed();
        }
        self.redraw_cursors(before);
    }
}
