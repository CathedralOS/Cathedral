//! Attenuation and hosting requests. Physical-source identity is never app-supplied.
use super::*;
use cathedral_contracts::{input as key, interaction as s};
use cathedral_user_runtime::server::Request;
impl Compositor {
    pub(super) fn session_request(&mut self, r: &Request, w: [u64; 6]) -> Result<[u64; 6], u64> {
        if r.control {
            return match w {
                [s::FOCUS_GRANT, client @ 0..=2, mask, delegate @ 0..=1, 0, 0] => self
                    .focus
                    .root_grant(
                        client as usize,
                        mask,
                        delegate == 1,
                        &self.tree,
                        self.clients,
                    )
                    .map(token),
                [s::HOST_GRANT, host @ 0..=2, child @ 0..=2, 0, 0, 0] => {
                    let grant = self
                        .tree
                        .grant(host as usize, child as usize, self.clients)?;
                    self.focus.revoke_branch(child as usize, &self.tree);
                    self.layout_changed();
                    Ok(token(grant))
                }
                [s::FOCUS_STATE, 0, 0, 0, 0, 0] => Ok(self.focus.state()),
                [s::SESSION_CONTROL, 0, 0, 0, 0, 0] => Ok(token(core::mem::take(&mut self.action))),
                _ => Err(abi::DENIED),
            };
        }
        if r.client == Some(2) {
            return match w {
                [s::KEY_SOURCE, code, state, 0, 0, 0] if code <= 255 && state <= 255 => {
                    let event = key::Event::decode(&[code as u8, state as u8])
                        .ok_or(abi::INVALID_ARGUMENT)?;
                    self.focus.key(event, &self.tree, self.clients);
                    Ok([0; 6])
                }
                [s::SESSION_CONTROL, action @ 1..=3, 0, 0, 0, 0] => {
                    self.action = action;
                    Ok([0; 6])
                }
                _ => Err(abi::DENIED),
            };
        }
        let client = r
            .client
            .and_then(app_index)
            .filter(|&c| self.clients[c].visible && self.clients[c].incarnation == r.incarnation)
            .ok_or(abi::DENIED)?;
        match w {
            [
                s::FOCUS_DELEGATE,
                grant,
                child @ 0..=2,
                mask,
                delegate @ 0..=1,
                0,
            ] => self
                .focus
                .delegate(
                    (client, grant),
                    child as usize,
                    mask,
                    delegate == 1,
                    &self.tree,
                    self.clients,
                )
                .map(token),
            [s::FOCUS_SELECT, grant, target @ 0..=2, 0, 0, 0] => self
                .focus
                .select(client, grant, target as usize, &self.tree, self.clients)
                .map(|()| [0; 6]),
            [s::FOCUS_REVOKE, grant, 0, 0, 0, 0] => self
                .focus
                .revoke(client, grant, &self.tree, self.clients)
                .map(|()| [0; 6]),
            [s::HOST_PLACE, grant, x, y, w, h] => {
                self.tree
                    .place(client, grant, Rect { x, y, w, h }, self.clients)?;
                self.layout_changed();
                Ok([0; 6])
            }
            [s::HOST_REVOKE, grant, 0, 0, 0, 0] => {
                let child = self.tree.revoke(client, grant, self.clients)?;
                self.focus.revoke_branch(child, &self.tree);
                for c in 0..3 {
                    if self.tree.within(child, c) {
                        self.pointers.scope_lost(c, self.clients);
                    }
                }
                self.layout_changed();
                Ok([0; 6])
            }
            _ => Err(abi::DENIED),
        }
    }
    pub(super) fn layout_changed(&mut self) {
        let old = core::array::from_fn::<_, 3, _>(|i| self.clients[i].scope);
        let cursors = self.pointers.cursor_bounds();
        self.tree.resolve(self.clients);
        // Invalidate hidden subtrees before clamping any surviving synthetic lease.
        for client in 0..3 {
            if !self.clients[client].scope.valid() {
                self.pointers.scope_lost(client, self.clients);
                self.focus.revoke_branch(client, &self.tree);
            }
        }
        for (client, previous) in old.iter().enumerate() {
            if *previous != self.clients[client].scope && self.clients[client].scope.valid() {
                self.pointers.moved(client, self.clients);
            }
        }
        for rect in old
            .into_iter()
            .chain(core::array::from_fn::<_, 3, _>(|i| self.clients[i].scope))
        {
            if rect.valid() {
                self.redraw(rect);
            }
        }
        self.redraw_cursors(cursors);
    }
}
fn token(value: u64) -> [u64; 6] {
    [0, value, 0, 0, 0, 0]
}
