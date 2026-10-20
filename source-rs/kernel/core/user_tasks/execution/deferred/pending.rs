//! Fixed pending sets. Owners enqueue only after accepting work; completion or
//! cancellation removes it before an account/launch can be reused.
use cathedral_kernel_contracts::user as abi;

#[derive(Clone, Copy)]
pub(in crate::user_tasks) enum Work {
    Admission,
    Memory,
    Cancel,
    Retire,
    Return,
}

#[derive(Default)]
pub(in crate::user_tasks) struct Pending {
    sets: [u32; 5],
    cursors: [usize; 5],
}

impl Pending {
    pub(in crate::user_tasks) fn insert(&mut self, kind: Work, slot: usize) {
        assert!(slot < abi::MAX_TASKS);
        self.sets[kind as usize] |= 1 << slot;
    }
    pub(in crate::user_tasks) fn remove(&mut self, kind: Work, slot: usize) {
        self.sets[kind as usize] &= !(1 << slot);
    }
    pub(in crate::user_tasks) fn any(&self) -> bool {
        self.sets.iter().any(|&set| set != 0)
    }
    pub(in crate::user_tasks) fn has(&self, kind: Work) -> bool {
        self.sets[kind as usize] != 0
    }
    /// Cancellation/reaping preserve increasing child order; newly enqueued
    /// descendants participate in the same drain, rather than a stale snapshot.
    pub(in crate::user_tasks) fn first(&mut self, kind: Work) -> Option<usize> {
        let set = self.sets[kind as usize];
        if set == 0 {
            return None;
        }
        let slot = set.trailing_zeros() as usize;
        self.remove(kind, slot);
        Some(slot)
    }
    /// Admission and memory work rotate so a low slot requeue cannot starve peers.
    pub(in crate::user_tasks) fn next(&mut self, kind: Work) -> Option<usize> {
        let index = kind as usize;
        let set = self.sets[index];
        if set == 0 {
            return None;
        }
        let after = set & (u32::MAX << self.cursors[index]);
        let slot = if after == 0 { set } else { after }.trailing_zeros() as usize;
        self.cursors[index] = (slot + 1) % abi::MAX_TASKS;
        self.remove(kind, slot);
        Some(slot)
    }
}
