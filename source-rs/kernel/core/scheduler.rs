//! Round-robin policy, independent of CPU context mechanics. The slot arena is
//! allocated at construction; admission and scheduling do not allocate.

use alloc::{vec, vec::Vec};

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
    cursor: usize,
}

impl Scheduler {
    pub fn new(limit: usize) -> Self {
        Self {
            states: vec![TaskState::Vacant; limit],
            generations: vec![0; limit],
            current: None,
            cursor: 0,
        }
    }
    pub fn current(&self) -> Option<usize> {
        self.current
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
        if matches!(event, Event::Timer) && !preempt && self.current.is_some() {
            self.wake(now);
            return self.current;
        }
        self.park(event, now);
        let count = self.states.len();
        for offset in 0..count {
            let candidate = (self.cursor + offset) % count;
            if self.states[candidate] == TaskState::Ready {
                self.states[candidate] = TaskState::Running;
                self.current = Some(candidate);
                self.cursor = (candidate + 1) % count;
                break;
            }
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
                Event::Timer | Event::Yield => TaskState::Ready,
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
mod tests {
    use super::*;
    #[test]
    fn cancelling_a_waiter_requires_reaping_before_reuse() {
        let mut scheduler = populated(2);
        let old = scheduler.id(0);
        scheduler.advance(Event::Yield, 0, true);
        scheduler.advance(Event::Block, 1, true);
        scheduler.park(Event::Yield, 2);
        scheduler.cancel(0);
        scheduler.unblock(0);
        assert!(!scheduler.is_alive(old));
        assert_eq!(scheduler.states()[0], TaskState::Exited);
        scheduler.reap(0);
        let new = scheduler.admit(0);
        assert_ne!(old, new);
        assert!(!scheduler.is_alive(old));
        assert!(scheduler.is_alive(new));
    }
    #[test]
    fn blocked_tasks_need_explicit_wakeup_and_keep_their_identity() {
        let mut scheduler = populated(2);
        let waiter = scheduler.id(0);
        scheduler.advance(Event::Yield, 0, true);
        assert_eq!(scheduler.advance(Event::Block, 1, true), Some(1));
        assert!(scheduler.is_alive(waiter));
        assert_eq!(scheduler.advance(Event::Timer, 100, true), Some(1));
        assert_eq!(scheduler.states()[0], TaskState::Blocked);
        scheduler.unblock(0);
        scheduler.unblock(0);
        assert_eq!(scheduler.advance(Event::Yield, 101, true), Some(0));
        scheduler.advance(Event::Exit, 102, true);
        scheduler.unblock(0);
        assert_eq!(scheduler.states()[0], TaskState::Exited);
    }
    fn populated(limit: usize) -> Scheduler {
        let mut scheduler = Scheduler::new(limit);
        for slot in 0..limit {
            scheduler.admit(slot);
        }
        scheduler
    }
    #[test]
    fn round_robin_and_cooperative_timer_have_distinct_behavior() {
        let mut scheduler = populated(2);
        assert_eq!(scheduler.advance(Event::Yield, 0, false), Some(0));
        assert_eq!(scheduler.advance(Event::Timer, 1, false), Some(0));
        assert_eq!(scheduler.advance(Event::Yield, 1, false), Some(1));
        assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
        assert_eq!(scheduler.states(), &[TaskState::Running, TaskState::Ready]);
    }
    #[test]
    fn sleeping_tasks_leave_cpu_idle_until_deadline_and_exits_are_terminal() {
        let mut scheduler = populated(2);
        scheduler.advance(Event::Yield, 0, false);
        assert_eq!(scheduler.advance(Event::Sleep(5), 0, false), Some(1));
        assert_eq!(scheduler.advance(Event::Sleep(10), 0, false), None);
        assert_eq!(scheduler.advance(Event::Timer, 4, true), None);
        assert_eq!(scheduler.advance(Event::Timer, 5, true), Some(0));
        assert_eq!(scheduler.advance(Event::Exit, 5, true), None);
        assert_eq!(scheduler.advance(Event::Timer, 10, true), Some(1));
        assert_eq!(scheduler.advance(Event::Exit, 10, true), None);
        assert!(scheduler.finished());
        assert_eq!(scheduler.advance(Event::Timer, 11, true), None);
    }
    #[test]
    fn zero_sleep_yields_and_deadlines_survive_clock_wrap() {
        let mut scheduler = populated(1);
        scheduler.advance(Event::Yield, u64::MAX - 2, true);
        assert_eq!(
            scheduler.advance(Event::Sleep(0), u64::MAX - 2, true),
            Some(0)
        );
        assert_eq!(scheduler.advance(Event::Sleep(5), u64::MAX - 2, true), None);
        assert_eq!(scheduler.advance(Event::Timer, 1, true), None);
        assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
        assert!(Scheduler::new(0).finished());
    }

    #[test]
    fn admission_requires_reaping_and_reused_slots_reject_stale_ids() {
        let mut scheduler = Scheduler::new(2);
        let first = scheduler.admit(scheduler.vacancy().unwrap());
        let survivor = scheduler.admit(scheduler.vacancy().unwrap());
        assert!(scheduler.vacancy().is_none());
        scheduler.advance(Event::Yield, 0, true);
        scheduler.park(Event::Exit, 1);
        assert!(!scheduler.is_alive(first));
        assert!(scheduler.vacancy().is_none());
        scheduler.reap(first.slot);
        let replacement = scheduler.admit(scheduler.vacancy().unwrap());
        assert_eq!(first.slot, replacement.slot);
        assert_ne!(first, replacement);
        assert!(!scheduler.is_alive(first));
        assert!(scheduler.is_alive(replacement));
        assert!(scheduler.is_alive(survivor));
        assert_eq!(
            scheduler.advance(Event::Yield, 1, true),
            Some(survivor.slot)
        );
    }
}
