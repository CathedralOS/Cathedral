//! Bounded waiter subscriptions and indexed deadlines, independent of contexts.
//! One wait per task; cancellation removes every subscription before slot reuse.
use crate::scheduler::deadlines::Deadlines;
use cathedral_kernel_contracts::user as abi;

#[derive(Clone, Copy)]
pub(in crate::user_tasks) enum Kind {
    Receive,
    Readiness,
    Child,
}

pub(in crate::user_tasks) struct Registry {
    active: [u32; 3],
    children: [u32; abi::MAX_LAUNCHES],
    child_of: [Option<usize>; abi::MAX_TASKS],
    changed_children: u32,
    deadlines: Deadlines,
    last: Option<u64>,
    ticks: u128,
}

impl Registry {
    #[cfg(test)]
    pub(in crate::user_tasks) fn new(tasks: usize) -> Self {
        Self::try_new(tasks).unwrap()
    }
    pub(in crate::user_tasks) fn try_new(
        tasks: usize,
    ) -> Result<Self, alloc::collections::TryReserveError> {
        assert!(tasks <= abi::MAX_TASKS);
        Ok(Self {
            active: [0; 3],
            children: [0; abi::MAX_LAUNCHES],
            child_of: [None; abi::MAX_TASKS],
            changed_children: 0,
            deadlines: Deadlines::try_new(tasks)?,
            last: None,
            ticks: 0,
        })
    }

    pub(in crate::user_tasks) fn register(
        &mut self,
        slot: usize,
        kind: Kind,
        deadline: Option<u64>,
        now: u64,
    ) {
        assert_eq!(self.active.iter().fold(0, |a, b| a | b) & (1 << slot), 0);
        self.advance(now);
        self.active[kind as usize] |= 1 << slot;
        if let Some(deadline) = deadline {
            let delay = if crate::user_tasks::time::reached(now, deadline) {
                0
            } else {
                deadline.wrapping_sub(now)
            };
            assert!(delay < (1 << 63));
            self.deadlines.set(slot, self.ticks + u128::from(delay));
        }
    }

    pub(in crate::user_tasks) fn watch_child(&mut self, slot: usize, launch: usize) {
        assert_ne!(self.active[Kind::Child as usize] & (1 << slot), 0);
        assert!(self.child_of[slot].is_none());
        self.children[launch] |= 1 << slot;
        self.child_of[slot] = Some(launch);
    }

    pub(in crate::user_tasks) fn child_changed(&mut self, launch: usize) {
        self.changed_children |= self.children[launch];
    }

    pub(in crate::user_tasks) fn remove(&mut self, slot: usize) {
        for active in &mut self.active {
            *active &= !(1 << slot);
        }
        self.deadlines.remove(slot);
        if let Some(launch) = self.child_of[slot].take() {
            self.children[launch] &= !(1 << slot);
        }
        self.changed_children &= !(1 << slot);
    }

    pub(super) fn candidates(&mut self, now: u64, ipc: u32, devices: u32) -> [u32; 3] {
        self.advance(now);
        let mut due = 0;
        while let Some(slot) = self.deadlines.pop_due(self.ticks) {
            due |= 1 << slot;
        }
        [
            (ipc | due) & self.active[Kind::Receive as usize],
            (ipc | devices | due) & self.active[Kind::Readiness as usize],
            (core::mem::take(&mut self.changed_children) | due) & self.active[Kind::Child as usize],
        ]
    }

    fn advance(&mut self, now: u64) {
        if let Some(last) = self.last.replace(now) {
            let elapsed = now.wrapping_sub(last);
            assert!(elapsed < (1 << 63), "wait clock comparison range exceeded");
            self.ticks += u128::from(elapsed);
        }
    }
}
