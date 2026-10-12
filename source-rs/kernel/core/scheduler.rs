//! Task state and scheduling entrance: wake, park, select a turn, resume.
//! shares.rs subdivides turns down the ownership tree; budget.rs gates eligibility
//! using measured CPU ceilings. CPU context mechanics stay outside policy; all
//! scheduling storage is allocated at construction.

use alloc::{vec, vec::Vec};
mod budget;
mod shares;
use budget::Budgets;
pub use budget::{CpuReport, PERIOD_US as CPU_PERIOD_US};
use shares::Shares;

/// A slot and generation within one runtime session. Reuse never revives an ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId {
    pub(crate) slot: usize,
    generation: u64,
}

impl TaskId {
    pub fn slot(self) -> usize {
        self.slot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Vacant,
    Ready,
    Running,
    Blocked,
    Sleeping { deadline: u64 },
    Exited,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// A nonblocking operation completed; retain the current timer quantum.
    Continue,
    Timer,
    Yield,
    Sleep(u64),
    Block,
    Exit,
}

pub struct Scheduler {
    states: Vec<TaskState>,
    generations: Vec<u64>,
    current: Option<usize>,
    shares: Shares,
    budgets: Budgets,
}

impl Scheduler {
    pub fn new(limit: usize) -> Self {
        Self {
            states: vec![TaskState::Vacant; limit],
            generations: vec![0; limit],
            current: None,
            shares: Shares::new(limit),
            budgets: Budgets::new(limit),
        }
    }
    /// Attach a dormant launch account to its current owner. The bounded launch
    /// tree requires increasing slot order. Cursor state survives loans/restarts.
    pub fn set_parent(&mut self, child: usize, parent: usize) {
        assert_eq!(self.states[child], TaskState::Vacant);
        assert!(parent < child);
        self.shares.set_parent(child, parent);
        self.budgets.refresh(&self.shares);
    }
    /// Boot configuration only. Zero inherits ancestor ceilings; it adds no cap.
    pub fn configure_cpu(&mut self, limits: &[u64]) -> bool {
        self.budgets.configure(limits)
    }
    /// Account the interval since the preceding sample to the running process.
    pub fn account_cpu(&mut self, now_us: u64) {
        self.budgets.advance(now_us, self.current, &self.shares);
    }
    /// Boot-stack work is charged explicitly to its requester or retired account.
    pub fn charge_cpu(&mut self, owner: usize, now_us: u64) {
        assert!(self.current.is_none());
        self.budgets.advance(now_us, Some(owner), &self.shares);
    }
    pub fn cpu_report(&self, slot: usize) -> CpuReport {
        self.budgets.report(slot)
    }
    pub fn has_eligible_ready(&self) -> bool {
        self.states
            .iter()
            .enumerate()
            .any(|(slot, state)| *state == TaskState::Ready && self.budgets.eligible[slot])
    }
    pub fn current(&self) -> Option<usize> {
        self.current
    }
    pub fn parent(&self, slot: usize) -> Option<usize> {
        self.shares.parent(slot)
    }
    pub fn states(&self) -> &[TaskState] {
        &self.states
    }
    pub fn finished(&self) -> bool {
        self.states
            .iter()
            .all(|state| matches!(state, TaskState::Vacant | TaskState::Exited))
    }

    pub fn vacancy(&self) -> Option<usize> {
        self.states
            .iter()
            .enumerate()
            .position(|(i, state)| *state == TaskState::Vacant && self.generations[i] < u64::MAX)
    }
    pub fn admit(&mut self, slot: usize) -> TaskId {
        assert_eq!(self.states[slot], TaskState::Vacant);
        self.generations[slot] = self.generations[slot]
            .checked_add(1)
            .expect("task generation exhausted");
        self.states[slot] = TaskState::Ready;
        self.id(slot)
    }
    pub fn id(&self, slot: usize) -> TaskId {
        assert_ne!(self.states[slot], TaskState::Vacant);
        TaskId {
            slot,
            generation: self.generations[slot],
        }
    }
    pub fn is_alive(&self, id: TaskId) -> bool {
        self.generations.get(id.slot) == Some(&id.generation)
            && matches!(
                self.states[id.slot],
                TaskState::Ready
                    | TaskState::Running
                    | TaskState::Sleeping { .. }
                    | TaskState::Blocked
            )
    }
    /// Release only after the runtime has retired context and stack storage.
    pub fn reap(&mut self, slot: usize) {
        assert_eq!(self.states[slot], TaskState::Exited);
        assert_ne!(self.current, Some(slot));
        self.states[slot] = TaskState::Vacant;
    }

    /// Sleep durations must be below half the wrapping clock's range.
    pub fn advance(&mut self, event: Event, now: u64, preempt: bool) -> Option<usize> {
        if (matches!(event, Event::Continue) || (matches!(event, Event::Timer) && !preempt))
            && self.current.is_some_and(|slot| self.budgets.eligible[slot])
        {
            self.wake(now);
            return self.current;
        }
        self.park(event, now);
        if let Some(candidate) = self.shares.select(&self.states, &self.budgets.eligible) {
            self.states[candidate] = TaskState::Running;
            self.current = Some(candidate);
        }
        self.current
    }

    pub fn unblock(&mut self, slot: usize) {
        if self.states[slot] == TaskState::Blocked {
            self.states[slot] = TaskState::Ready;
        }
    }

    /// Cancel a non-running task; the runtime must reclaim it before reaping.
    pub fn cancel(&mut self, slot: usize) {
        assert_ne!(self.current, Some(slot));
        assert!(matches!(
            self.states[slot],
            TaskState::Ready | TaskState::Blocked | TaskState::Sleeping { .. }
        ));
        self.states[slot] = TaskState::Exited;
    }

    fn wake(&mut self, now: u64) {
        for state in &mut self.states {
            if let TaskState::Sleeping { deadline } = *state
                && now.wrapping_sub(deadline) < (1 << 63)
            {
                *state = TaskState::Ready;
            }
        }
    }

    /// Suspend to the boot context for admission/reaping, without selecting a task.
    pub fn park(&mut self, event: Event, now: u64) {
        self.wake(now);
        if let Some(current) = self.current {
            self.states[current] = match event {
                Event::Continue | Event::Timer | Event::Yield => TaskState::Ready,
                Event::Sleep(delay) => {
                    assert!(delay < (1 << 63), "sleep exceeds clock comparison range");
                    if delay == 0 {
                        TaskState::Ready
                    } else {
                        TaskState::Sleeping {
                            deadline: now.wrapping_add(delay),
                        }
                    }
                }
                Event::Exit => TaskState::Exited,
                Event::Block => TaskState::Blocked,
            };
        }
        self.current = None;
    }
}

#[cfg(test)]
mod tests;
