//! Allocation-free round-robin policy, independent of CPU context mechanics.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Sleeping { deadline: u64 },
    Exited,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Timer,
    Yield,
    Sleep(u64),
    Exit,
}

pub struct Scheduler<const N: usize> {
    states: [TaskState; N],
    current: Option<usize>,
    cursor: usize,
}

impl<const N: usize> Default for Scheduler<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Scheduler<N> {
    pub const fn new() -> Self {
        Self {
            states: [TaskState::Ready; N],
            current: None,
            cursor: 0,
        }
    }
    pub fn current(&self) -> Option<usize> {
        self.current
    }
    pub fn states(&self) -> &[TaskState; N] {
        &self.states
    }
    pub fn finished(&self) -> bool {
        self.states.iter().all(|state| *state == TaskState::Exited)
    }

    /// Sleep durations must be below half the wrapping clock's range.
    pub fn advance(&mut self, event: Event, now: u64, preempt: bool) -> Option<usize> {
        for state in &mut self.states {
            if let TaskState::Sleeping { deadline } = *state
                && now.wrapping_sub(deadline) < (1 << 63)
            {
                *state = TaskState::Ready;
            }
        }
        if let Some(current) = self.current {
            if matches!(event, Event::Timer) && !preempt {
                return Some(current);
            }
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
            };
        }
        self.current = None;
        for offset in 0..N {
            let candidate = (self.cursor + offset) % N;
            if self.states[candidate] == TaskState::Ready {
                self.states[candidate] = TaskState::Running;
                self.current = Some(candidate);
                self.cursor = (candidate + 1) % N;
                break;
            }
        }
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_robin_and_cooperative_timer_have_distinct_behavior() {
        let mut scheduler = Scheduler::<2>::new();
        assert_eq!(scheduler.advance(Event::Yield, 0, false), Some(0));
        assert_eq!(scheduler.advance(Event::Timer, 1, false), Some(0));
        assert_eq!(scheduler.advance(Event::Yield, 1, false), Some(1));
        assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
        assert_eq!(scheduler.states(), &[TaskState::Running, TaskState::Ready]);
    }
    #[test]
    fn sleeping_tasks_leave_cpu_idle_until_deadline_and_exits_are_terminal() {
        let mut scheduler = Scheduler::<2>::new();
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
        let mut scheduler = Scheduler::<1>::new();
        scheduler.advance(Event::Yield, u64::MAX - 2, true);
        assert_eq!(
            scheduler.advance(Event::Sleep(0), u64::MAX - 2, true),
            Some(0)
        );
        assert_eq!(scheduler.advance(Event::Sleep(5), u64::MAX - 2, true), None);
        assert_eq!(scheduler.advance(Event::Timer, 1, true), None);
        assert_eq!(scheduler.advance(Event::Timer, 2, true), Some(0));
        assert!(Scheduler::<0>::new().finished());
    }
}
