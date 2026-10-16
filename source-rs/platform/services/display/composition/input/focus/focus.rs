// Attenuated keyboard routing grants. Active focus never escapes its authorized subtree.
use crate::composition::scene::{Client, hosting::Tree};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display::interaction as wire;
use cathedral_platform_contracts::input;
#[derive(Clone, Copy)]
struct Grant {
    token: u64,
    incarnation: u64,
    parent: Option<usize>,
    mask: u64,
    delegate: bool,
}
#[derive(Clone, Copy)]
struct Queue {
    events: [[u64; 6]; 8],
    len: usize,
}
// Constant array initialization avoids eight nested debug Default calls; the
// unoptimized compositor must fit its guarded stack and bounded loaded image.
#[allow(clippy::derivable_impls)]
impl Default for Queue {
    fn default() -> Self {
        Self {
            events: [[0; 6]; 8],
            len: 0,
        }
    }
}
pub struct Focus {
    grants: [Option<Grant>; 3],
    queues: [Queue; 3],
    active: Option<usize>,
    epoch: u64,
    held: u64,
    suppressed: u64,
}
impl Default for Focus {
    fn default() -> Self {
        Self::empty()
    }
}
impl Focus {
    pub const fn empty() -> Self {
        Self {
            grants: [None; 3],
            queues: [Queue {
                events: [[0; 6]; 8],
                len: 0,
            }; 3],
            active: None,
            epoch: 0,
            held: 0,
            suppressed: 0,
        }
    }
    fn mint(
        &mut self,
        client: usize,
        parent: Option<usize>,
        mask: u64,
        delegate: bool,
        clients: &[Client; 3],
    ) -> Result<u64, u64> {
        if client >= 3 || mask == 0 || mask & !wire::KEY_MASK != 0 || !clients[client].visible {
            return Err(abi::DENIED);
        }
        self.epoch = self.epoch.checked_add(1).ok_or(abi::DENIED)?;
        self.grants[client] = Some(Grant {
            token: self.epoch,
            incarnation: clients[client].incarnation,
            parent,
            mask,
            delegate,
        });
        Ok(self.epoch)
    }
    pub fn root_grant(
        &mut self,
        client: usize,
        mask: u64,
        delegate: bool,
        tree: &Tree,
        clients: &[Client; 3],
    ) -> Result<u64, u64> {
        if client >= 3 || tree.parent(client).is_some() {
            return Err(abi::DENIED);
        }
        self.revoke_branch(client, tree);
        self.mint(client, None, mask, delegate, clients)
    }
    fn owned(&self, client: usize, token: u64, clients: &[Client; 3]) -> Result<Grant, u64> {
        self.grants
            .get(client)
            .copied()
            .flatten()
            .filter(|g| g.token == token && g.incarnation == clients[client].incarnation)
            .ok_or(abi::DENIED)
    }
    pub fn delegate(
        &mut self,
        authority: (usize, u64),
        child: usize,
        mask: u64,
        delegate: bool,
        tree: &Tree,
        clients: &[Client; 3],
    ) -> Result<u64, u64> {
        let (host, token) = authority;
        let grant = self.owned(host, token, clients)?;
        if child >= 3
            || tree.parent(child) != Some(host)
            || !grant.delegate
            || mask & !grant.mask != 0
        {
            return Err(abi::DENIED);
        }
        self.revoke_branch(child, tree);
        self.mint(child, Some(host), mask, delegate, clients)
    }
    fn valid(&self, mut client: usize, tree: &Tree, clients: &[Client; 3]) -> bool {
        for _ in 0..3 {
            let Some(g) = self.grants[client] else {
                return false;
            };
            if g.incarnation != clients[client].incarnation
                || !clients[client].visible
                || !clients[client].scope.valid()
                || g.parent != tree.parent(client)
            {
                return false;
            }
            match g.parent {
                None => return true,
                Some(parent) => client = parent,
            }
        }
        false
    }
    pub fn select(
        &mut self,
        caller: usize,
        token: u64,
        target: usize,
        tree: &Tree,
        clients: &[Client; 3],
    ) -> Result<(), u64> {
        let grant = self.owned(caller, token, clients)?;
        if target >= 3
            || !tree.within(caller, target)
            || (target != caller && !grant.delegate)
            || !self.valid(target, tree, clients)
            || self
                .active
                .is_none_or(|active| !tree.within(caller, active))
        {
            return Err(abi::DENIED);
        }
        self.switch(Some(target));
        Ok(())
    }
    pub fn click(&mut self, target: Option<usize>, tree: &Tree, clients: &[Client; 3]) {
        let mut target = target;
        while let Some(client) = target {
            if self.valid(client, tree, clients) {
                break;
            }
            target = tree.parent(client);
        }
        self.switch(target);
    }
    pub fn revoke(
        &mut self,
        client: usize,
        token: u64,
        tree: &Tree,
        clients: &[Client; 3],
    ) -> Result<(), u64> {
        self.owned(client, token, clients)?;
        self.revoke_branch(client, tree);
        Ok(())
    }
    pub fn revoke_branch(&mut self, client: usize, tree: &Tree) {
        if self
            .active
            .is_some_and(|active| tree.within(client, active))
        {
            self.switch(None);
        }
        for descendant in 0..3 {
            if tree.within(client, descendant) && self.grants[descendant].is_some() {
                // Repeated invalidation must preserve the already queued barrier.
                self.clear(descendant);
                self.grants[descendant] = None;
            }
        }
    }
    pub fn state(&self) -> [u64; 6] {
        [
            0,
            self.active.map_or(0, |c| c as u64 + 1),
            self.active
                .and_then(|c| self.grants[c])
                .map_or(0, |g| g.token),
            0,
            0,
            0,
        ]
    }
}
