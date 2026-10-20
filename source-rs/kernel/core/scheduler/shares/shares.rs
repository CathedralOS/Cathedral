// Hierarchical turns over maintained active branches. State/gate changes update
// only ancestors whose runnable status changes. Each level visits active direct
// children, preserving slot-ordered rotation without scanning dormant accounts.
use alloc::{vec, vec::Vec};

#[derive(Clone, Copy)]
struct Account {
    parent: Option<usize>,
    cursor: usize,
    ready: bool,
    allowed: bool,
    active: bool,
    first: Option<usize>,
    previous: Option<usize>,
    next: Option<usize>,
}
impl Default for Account {
    fn default() -> Self {
        Self {
            parent: None,
            cursor: 0,
            ready: false,
            allowed: true,
            active: false,
            first: None,
            previous: None,
            next: None,
        }
    }
}

pub(super) struct Shares {
    accounts: Vec<Account>,
    root_cursor: usize,
    first: Option<usize>,
    #[cfg(test)]
    pub(super) inspections: usize,
}
impl Shares {
    pub(super) fn new(count: usize) -> Self {
        Self {
            accounts: vec![Account::default(); count],
            root_cursor: 0,
            first: None,
            #[cfg(test)]
            inspections: 0,
        }
    }
    pub(super) fn set_parent(&mut self, child: usize, parent: usize) {
        let old = self.accounts[child].parent;
        if self.accounts[child].active {
            self.unlink(child);
        }
        self.accounts[child].parent = Some(parent);
        if self.accounts[child].active {
            self.link(child);
        }
        if let Some(old) = old {
            self.refresh(old);
        }
        self.refresh(parent);
    }
    pub(super) fn parent(&self, child: usize) -> Option<usize> {
        self.accounts[child].parent
    }
    pub(super) fn ready(&mut self, slot: usize, ready: bool) {
        self.accounts[slot].ready = ready;
        self.refresh(slot);
    }
    pub(super) fn allow(&mut self, slot: usize, allowed: bool) {
        self.accounts[slot].allowed = allowed;
        self.refresh(slot);
    }
    pub(super) fn eligible(&self, mut slot: usize) -> bool {
        loop {
            if !self.accounts[slot].allowed {
                return false;
            }
            match self.parent(slot) {
                Some(parent) => slot = parent,
                None => return true,
            }
        }
    }
    pub(super) fn has_ready(&self) -> bool {
        self.first.is_some()
    }
    fn refresh(&mut self, mut slot: usize) {
        loop {
            let account = &self.accounts[slot];
            let active = account.allowed && (account.ready || account.first.is_some());
            if account.active == active {
                return;
            }
            self.accounts[slot].active = active;
            if active {
                self.link(slot);
            } else {
                self.unlink(slot);
            }
            match self.parent(slot) {
                Some(parent) => slot = parent,
                None => return,
            }
        }
    }
    fn link(&mut self, slot: usize) {
        let parent = self.parent(slot);
        let first = parent.map_or(self.first, |p| self.accounts[p].first);
        self.accounts[slot].previous = None;
        self.accounts[slot].next = first;
        if let Some(first) = first {
            self.accounts[first].previous = Some(slot);
        }
        if let Some(parent) = parent {
            self.accounts[parent].first = Some(slot);
        } else {
            self.first = Some(slot);
        }
    }
    fn unlink(&mut self, slot: usize) {
        let Account {
            parent,
            previous,
            next,
            ..
        } = self.accounts[slot];
        if let Some(previous) = previous {
            self.accounts[previous].next = next;
        } else if let Some(parent) = parent {
            self.accounts[parent].first = next;
        } else {
            self.first = next;
        }
        if let Some(next) = next {
            self.accounts[next].previous = previous;
        }
        self.accounts[slot].previous = None;
        self.accounts[slot].next = None;
    }
    pub(super) fn select(&mut self) -> Option<usize> {
        let mut parent = None;
        loop {
            let cursor = parent.map_or(self.root_cursor, |slot: usize| self.accounts[slot].cursor);
            let mut candidate = parent.filter(|&slot| self.accounts[slot].ready);
            let mut child = parent.map_or(self.first, |slot| self.accounts[slot].first);
            while let Some(slot) = child {
                #[cfg(test)]
                {
                    self.inspections += 1;
                }
                // Same cyclic slot order as the original table scan.
                let key = |slot| (slot < cursor, slot);
                if candidate.is_none_or(|old| key(slot) < key(old)) {
                    candidate = Some(slot);
                }
                child = self.accounts[slot].next;
            }
            let candidate = candidate?;
            let next = (candidate + 1) % self.accounts.len();
            if let Some(slot) = parent {
                self.accounts[slot].cursor = next;
            } else {
                self.root_cursor = next;
            }
            if parent == Some(candidate) {
                return Some(candidate);
            }
            parent = Some(candidate);
        }
    }
}
