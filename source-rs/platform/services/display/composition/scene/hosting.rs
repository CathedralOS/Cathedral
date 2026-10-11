//! Scene ownership across process boundaries. Hosts place children; root resolves clips.
use crate::composition::scene::{Client, Rect};
use cathedral_contracts::user as abi;
#[derive(Clone, Copy)]
struct Grant {
    token: u64,
    host: usize,
    host_inc: u64,
    child_inc: u64,
    placed: bool,
}
#[derive(Default)]
pub struct Tree {
    parents: [Option<usize>; 3],
    grants: [Option<Grant>; 3],
    local: [Rect; 3],
    epoch: u64,
}
impl Tree {
    pub fn parent(&self, child: usize) -> Option<usize> {
        self.parents[child]
    }
    pub fn within(&self, ancestor: usize, mut child: usize) -> bool {
        for _ in 0..3 {
            if child == ancestor {
                return true;
            }
            let Some(parent) = self.parents[child] else {
                return false;
            };
            child = parent;
        }
        false
    }
    pub fn place_root(&mut self, client: usize, rect: Rect) -> Result<(), u64> {
        if self.parents[client].is_some() {
            return Err(abi::DENIED);
        }
        self.local[client] = rect;
        Ok(())
    }
    pub fn grant(&mut self, host: usize, child: usize, clients: &[Client; 3]) -> Result<u64, u64> {
        // Fixed stack order: ancestors precede children. This also excludes cycles.
        if host >= child || child >= 3 || !clients[host].visible || !clients[child].visible {
            return Err(abi::DENIED);
        }
        if self.parents[child].is_some_and(|p| p != host) {
            return Err(abi::DENIED);
        }
        self.epoch = self.epoch.checked_add(1).ok_or(abi::DENIED)?;
        self.parents[child] = Some(host);
        self.grants[child] = Some(Grant {
            token: self.epoch,
            host,
            host_inc: clients[host].incarnation,
            child_inc: clients[child].incarnation,
            placed: false,
        });
        Ok(self.epoch)
    }
    fn owned(&self, host: usize, token: u64, clients: &[Client; 3]) -> Result<usize, u64> {
        self.grants
            .iter()
            .position(|g| {
                g.is_some_and(|g| {
                    g.host == host && g.token == token && g.host_inc == clients[host].incarnation
                })
            })
            .filter(|&child| self.grants[child].unwrap().child_inc == clients[child].incarnation)
            .ok_or(abi::DENIED)
    }
    pub fn place(
        &mut self,
        host: usize,
        token: u64,
        rect: Rect,
        clients: &[Client; 3],
    ) -> Result<usize, u64> {
        let child = self.owned(host, token, clients)?;
        if !rect.valid() || rect.x >= 256 || rect.y >= 256 || rect.w > 256 || rect.h > 256 {
            return Err(abi::INVALID_ARGUMENT);
        }
        self.local[child] = rect;
        self.grants[child].as_mut().unwrap().placed = true;
        Ok(child)
    }
    pub fn revoke(&mut self, host: usize, token: u64, clients: &[Client; 3]) -> Result<usize, u64> {
        let child = self.owned(host, token, clients)?;
        self.lost(child);
        Ok(child)
    }
    pub fn lost(&mut self, client: usize) {
        for child in 0..3 {
            if self.within(client, child) {
                self.grants[child] = None;
            }
        }
    }
    pub fn resolve(&self, clients: &mut [Client; 3]) {
        for child in 0..3 {
            let local = self.local[child];
            clients[child].scope = match self.parents[child] {
                None => local,
                Some(parent) => {
                    let r = clients[parent].scope;
                    if !clients[parent].visible
                        || !self.grants[child].is_some_and(|g| {
                            g.placed
                                && g.host_inc == clients[parent].incarnation
                                && g.child_inc == clients[child].incarnation
                        })
                    {
                        Rect::default()
                    } else {
                        Rect {
                            x: r.x + local.x,
                            y: r.y + local.y,
                            w: local.w.min(r.w.saturating_sub(local.x)),
                            h: local.h.min(r.h.saturating_sub(local.y)),
                        }
                    }
                }
            };
        }
    }
}
