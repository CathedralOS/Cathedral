//! Attenuation and hosting requests. Physical-source identity is never app-supplied.
use super::*;
use cathedral_platform_contracts::display::interaction as s;
use cathedral_platform_contracts::input as key;
use cathedral_user_runtime::server::Request;
impl Compositor {
    pub(super) fn session_request(&mut self, r: &Request, w: [u64; 6]) -> Result<[u64; 6], u64> {
        if r.control {
            return match w {
                [s::FOCUS_GRANT, client @ 0..=2, mask, delegate @ 0..=1, 0, 0] => self
                    .composition
                    .focus
                    .root_grant(
                        client as usize,
                        mask,
                        delegate == 1,
                        &self.composition.tree,
                        self.composition.clients,
                    )
                    .map(token),
                [s::HOST_GRANT, host @ 0..=2, child @ 0..=2, 0, 0, 0] => {
                    let grant = self.composition.tree.grant(
                        host as usize,
                        child as usize,
                        self.composition.clients,
                    )?;
                    self.composition
                        .focus
                        .revoke_branch(child as usize, &self.composition.tree);
                    self.layout_changed();
                    Ok(token(grant))
                }
                [s::FOCUS_STATE, 0, 0, 0, 0, 0] => Ok(self.composition.focus.state()),
                [s::SESSION_CONTROL, 0, 0, 0, 0, 0] => Ok(token(core::mem::take(&mut self.action))),
                _ => Err(abi::DENIED),
            };
        }
        if r.client == Some(2) {
            return match w {
                [s::KEY_SOURCE, code, state, 0, 0, 0] if code <= 255 && state <= 255 => {
                    let event = key::Event::decode(&[code as u8, state as u8])
                        .ok_or(abi::INVALID_ARGUMENT)?;
                    self.composition.focus.key(
                        event,
                        &self.composition.tree,
                        self.composition.clients,
                    );
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
            .filter(|&c| {
                self.composition.clients[c].visible
                    && self.composition.clients[c].incarnation == r.incarnation
            })
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
                .composition
                .focus
                .delegate(
                    (client, grant),
                    child as usize,
                    mask,
                    delegate == 1,
                    &self.composition.tree,
                    self.composition.clients,
                )
                .map(token),
            [s::FOCUS_SELECT, grant, target @ 0..=2, 0, 0, 0] => self
                .composition
                .focus
                .select(
                    client,
                    grant,
                    target as usize,
                    &self.composition.tree,
                    self.composition.clients,
                )
                .map(|()| [0; 6]),
            [s::FOCUS_REVOKE, grant, 0, 0, 0, 0] => self
                .composition
                .focus
                .revoke(
                    client,
                    grant,
                    &self.composition.tree,
                    self.composition.clients,
                )
                .map(|()| [0; 6]),
            [s::HOST_PLACE, grant, x, y, w, h] => {
                self.composition.tree.place(
                    client,
                    grant,
                    Rect { x, y, w, h },
                    self.composition.clients,
                )?;
                self.layout_changed();
                Ok([0; 6])
            }
            [s::HOST_REVOKE, grant, 0, 0, 0, 0] => {
                let child =
                    self.composition
                        .tree
                        .revoke(client, grant, self.composition.clients)?;
                self.composition
                    .focus
                    .revoke_branch(child, &self.composition.tree);
                for c in 0..3 {
                    if self.composition.tree.within(child, c) {
                        self.composition
                            .pointers
                            .scope_lost(c, self.composition.clients);
                    }
                }
                self.layout_changed();
                Ok([0; 6])
            }
            _ => Err(abi::DENIED),
        }
    }
    pub(super) fn layout_changed(&mut self) {
        let old = core::array::from_fn::<_, 3, _>(|i| self.composition.clients[i].scope);
        let cursors = self.composition.pointers.cursor_bounds();
        self.composition.tree.resolve(self.composition.clients);
        // Invalidate hidden subtrees before clamping any surviving synthetic lease.
        for client in 0..3 {
            if !self.composition.clients[client].scope.valid() {
                self.composition
                    .pointers
                    .scope_lost(client, self.composition.clients);
                self.composition
                    .focus
                    .revoke_branch(client, &self.composition.tree);
            }
        }
        for (client, previous) in old.iter().enumerate() {
            if *previous != self.composition.clients[client].scope
                && self.composition.clients[client].scope.valid()
            {
                self.composition
                    .pointers
                    .moved(client, self.composition.clients);
            }
        }
        for rect in old.into_iter().chain(core::array::from_fn::<_, 3, _>(|i| {
            self.composition.clients[i].scope
        })) {
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
