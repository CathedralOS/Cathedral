//! Connection-authorized pointer ingress, client delivery and cursor damage.
use super::*;
use cathedral_platform_contracts::display::pointer as p;
use cathedral_user_runtime::server::Request;
impl Compositor {
    pub(super) fn pointer_request(&mut self, r: &Request, w: [u64; 6]) -> Result<[u64; 6], u64> {
        let before = self.composition.pointers.cursor_bounds();
        let result = if r.control {
            match w {
                [p::LEASE, client @ 0..=2, grant @ 0..=1, 0, 0, 0] => self
                    .composition
                    .pointers
                    .lease(client as usize, grant == 1, self.composition.clients)
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
                [p::STATE, id @ 0..=1, 0, 0, 0, 0] => {
                    Ok(self.composition.pointers.state(id as usize))
                }
                _ => Err(abi::DENIED),
            }
        } else if r.client == Some(2) {
            match w {
                [p::DEVICE, dx, dy, buttons @ 0..=7, reset @ 0..=1, 0] => {
                    self.device_report(
                        r.incarnation,
                        dx as i64,
                        dy as i64,
                        buttons as u8,
                        reset == 1,
                    );
                    Ok([0; 6])
                }
                [p::DEVICE_BATCH, ..] => {
                    // Validate every report before any state or pixel mutation.
                    let batch = p::SourceBatch::decode(w).ok_or(abi::INVALID_ARGUMENT)?;
                    for report in batch.reports() {
                        self.device_report(
                            r.incarnation,
                            i64::from(report.dx),
                            i64::from(report.dy),
                            report.buttons,
                            report.reset,
                        );
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
                    c < 3
                        && self.composition.clients[c].visible
                        && self.composition.clients[c].incarnation == r.incarnation
                })
                .ok_or(abi::DENIED)?;
            match w {
                [p::NEXT, 0, 0, 0, 0, 0] => Ok(self.composition.pointers.next(client).words()),
                [p::CAPTURE, id @ 0..=1, take @ 0..=1, 0, 0, 0] => self
                    .composition
                    .pointers
                    .capture(client, id as usize, take == 1)
                    .map(|()| [0; 6]),
                [p::DRIVE, epoch, x, y, buttons @ 0..=7, 0] => self
                    .composition
                    .pointers
                    .drive(client, epoch, x, y, buttons as u8, self.composition.clients)
                    .map(|()| [0; 6]),
                _ => Err(abi::DENIED),
            }
        };
        // Polling/capture/state do not change cursor pixels.
        if matches!(w[0], p::DEVICE | p::DEVICE_BATCH | p::DRIVE | p::LEASE) {
            self.redraw_cursors(before);
        }
        result
    }
    fn device_report(&mut self, incarnation: u64, dx: i64, dy: i64, buttons: u8, reset: bool) {
        if self.source != incarnation {
            self.composition
                .pointers
                .source_lost(self.composition.clients);
            self.source = incarnation;
        }
        let previous = self.composition.pointers.state(0)[3];
        self.composition
            .pointers
            .device(dx, dy, buttons, reset, self.composition.clients);
        if reset {
            self.composition.focus.source_lost();
        } else if previous == 0 && self.composition.pointers.state(0)[3] != 0 {
            let state = self.composition.pointers.state(0);
            let target = self
                .composition
                .clients
                .iter()
                .rposition(|c| c.hit(state[1], state[2]));
            self.composition
                .focus
                .click(target, &self.composition.tree, self.composition.clients);
        }
    }
    pub(super) fn redraw_cursors(&mut self, before: [Rect; 2]) {
        for rect in before
            .into_iter()
            .chain(self.composition.pointers.cursor_bounds())
        {
            self.redraw(rect);
        }
    }
    pub(super) fn disconnected(&mut self, client: usize, incarnation: u64) {
        let before = self.composition.pointers.cursor_bounds();
        if client == 2 {
            if self.source == incarnation {
                self.composition
                    .pointers
                    .source_lost(self.composition.clients);
                self.composition.focus.source_lost();
                self.source = 0;
                cathedral_user_runtime::write(
                    b"Cathedral pointer: source disconnected; capture cancelled\n",
                )
                .unwrap();
            }
        } else if let Some(client) = app_index(client)
            && self.composition.clients[client].incarnation == incarnation
        {
            self.pending[client] = None;
            self.composition
                .focus
                .revoke_branch(client, &self.composition.tree);
            self.composition.tree.lost(client);
            for descendant in 0..3 {
                if self.composition.tree.within(client, descendant) {
                    self.composition
                        .pointers
                        .scope_lost(descendant, self.composition.clients);
                }
            }
            self.composition
                .pointers
                .client_lost(client, self.composition.clients);
            self.composition.clients[client].disconnect(incarnation);
            self.layout_changed();
        }
        self.redraw_cursors(before);
    }
}
