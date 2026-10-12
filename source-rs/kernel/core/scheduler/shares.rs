//! Work-conserving hierarchical turns. At each level, a runnable process and
//! each runnable direct-child subtree get one turn in rotation. Descendants
//! subdivide that turn; they cannot add contenders at their parent's outer level.
//! A yield/block forfeits the rest of a turn, with no credit bank or cursor reset.
use super::TaskState;
use alloc::{vec, vec::Vec};

#[derive(Clone, Copy, Default)]
struct Account {
    parent: Option<usize>,
    cursor: usize,
    runnable: bool,
}
pub(super) struct Shares {
    accounts: Vec<Account>,
    root_cursor: usize,
}
impl Shares {
    pub(super) fn new(count: usize) -> Self {
        Self {
            accounts: vec![Account::default(); count],
            root_cursor: 0,
        }
    }
    pub(super) fn set_parent(&mut self, child: usize, parent: usize) {
        self.accounts[child].parent = Some(parent);
    }
    pub(super) fn parent(&self, child: usize) -> Option<usize> {
        self.accounts[child].parent
    }
    pub(super) fn select(&mut self, states: &[TaskState], eligible: &[bool]) -> Option<usize> {
        // Parents precede children, so one reverse pass summarizes readiness.
        for (slot, account) in self.accounts.iter_mut().enumerate() {
            account.runnable = states[slot] == TaskState::Ready && eligible[slot];
        }
        for child in (0..self.accounts.len()).rev() {
            if self.accounts[child].runnable
                && let Some(parent) = self.accounts[child].parent
            {
                self.accounts[parent].runnable = true;
            }
        }
        let mut parent = None;
        loop {
            let cursor = parent.map_or(self.root_cursor, |slot: usize| self.accounts[slot].cursor);
            let count = self.accounts.len();
            let candidate = (0..count)
                .map(|offset| (cursor + offset) % count)
                .find(|&slot| {
                    if parent == Some(slot) {
                        states[slot] == TaskState::Ready && eligible[slot]
                    } else {
                        self.accounts[slot].parent == parent && self.accounts[slot].runnable
                    }
                })?;
            let next = (candidate + 1) % count;
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

#[cfg(test)]
mod tests;
