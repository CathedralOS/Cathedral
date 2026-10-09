//! Single-CPU execution and lifetime management. No allocation or reclamation
//! in schedule(); admission and reaping happen only after restoring boot RSP.

use super::{Config, SpawnError, stacks};
use crate::{
    extent::FrameAllocator,
    scheduler::{Event, Scheduler, TaskId, TaskState},
};
use alloc::{alloc::alloc, boxed::Box, vec::Vec};
use cathedral_arch::{self as arch, Context, SwitchCause};
use core::{
    alloc::Layout,
    sync::atomic::{AtomicPtr, Ordering},
};

#[derive(Clone, Debug)]
pub struct RunStats {
    pub switches: u64,
    pub preemptions: [u64; arch::MAX_TASK_SLOTS],
    pub spawned: usize,
    pub exits: usize,
}
impl Default for RunStats {
    fn default() -> Self {
        Self {
            switches: 0,
            preemptions: [0; arch::MAX_TASK_SLOTS],
            spawned: 0,
            exits: 0,
        }
    }
}

pub(super) enum Request {
    Schedule(Event),
    Spawn(fn()),
}
pub(super) struct Task {
    context: Context,
    entry: fn(),
    pub reply: Option<Result<TaskId, SpawnError>>,
}
impl Task {
    fn allocate(context: Context, entry: fn()) -> Result<Box<Self>, SpawnError> {
        // SAFETY: Matching global allocator/layout; initialize the complete value
        // before transferring sole ownership to Box. A null allocation is ordinary
        // admission failure, not the infallible allocator's terminal OOM path.
        unsafe {
            let pointer = alloc(Layout::new::<Self>()).cast::<Self>();
            if pointer.is_null() {
                return Err(SpawnError::OutOfHeap);
            }
            pointer.write(Self {
                context,
                entry,
                reply: None,
            });
            Ok(Box::from_raw(pointer))
        }
    }
}
pub(super) struct Session {
    pub scheduler: Scheduler,
    pub tasks: Vec<Option<Box<Task>>>,
    pub frames: *mut FrameAllocator,
    pub pending: Request,
    boot: Context,
    service: Option<(usize, fn())>,
    config: Config,
    baseline: usize,
    stats: RunStats,
}
static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

impl Session {
    unsafe fn spawn(&mut self, entry: fn()) -> Result<TaskId, SpawnError> {
        let slot = self.scheduler.vacancy().ok_or(SpawnError::TaskLimit)?;
        // SAFETY: Borrowed exclusively for run(), with IRQs off on the boot stack.
        let frames = unsafe { &mut *self.frames };
        let budget = self
            .config
            .frame_limit
            .saturating_sub(frames.allocated() - self.baseline);
        // SAFETY: Vacant slot, unique physical custody and exclusive mapper access.
        let stack = unsafe { stacks::allocate(frames, slot, budget)? };
        // SAFETY: Fresh guarded stack remains live until the task has switched out.
        let context = unsafe { Context::new(stack, task_entry, slot) };
        self.tasks[slot] = match Task::allocate(context, entry) {
            Ok(task) => Some(task),
            Err(error) => {
                // SAFETY: Admission failed; this stack was never entered.
                unsafe {
                    stacks::release(frames, slot);
                }
                return Err(error);
            }
        };
        let id = self.scheduler.admit(slot);
        self.stats.spawned += 1;
        Ok(id)
    }

    unsafe fn service(&mut self) {
        for slot in 0..self.tasks.len() {
            if self.scheduler.states()[slot] == TaskState::Exited {
                drop(self.tasks[slot].take());
                // SAFETY: Running on boot stack; no context can resume this task.
                unsafe {
                    stacks::release(&mut *self.frames, slot);
                }
                self.scheduler.reap(slot);
            }
        }
        if let Some((caller, entry)) = self.service.take() {
            // SAFETY: Serialized admission on boot stack, never inside the IRQ.
            let result = unsafe { self.spawn(entry) };
            self.tasks[caller].as_mut().unwrap().reply = Some(result);
        }
    }
}

pub(super) unsafe fn run(
    frames: &mut FrameAllocator,
    config: Config,
    entries: &[fn()],
) -> Result<RunStats, SpawnError> {
    assert!(
        ACTIVE.load(Ordering::Acquire).is_null(),
        "nested task session"
    );
    if config.task_limit == 0 || config.task_limit > arch::MAX_TASK_SLOTS {
        return Err(SpawnError::InvalidLimit);
    }
    if entries.len() > config.task_limit {
        return Err(SpawnError::TaskLimit);
    }
    let mut session = Box::new(Session {
        scheduler: Scheduler::new(config.task_limit),
        tasks: (0..config.task_limit).map(|_| None).collect(),
        frames,
        pending: Request::Schedule(Event::Yield),
        boot: Context::default(),
        service: None,
        config,
        baseline: frames.allocated(),
        stats: RunStats::default(),
    });
    for &entry in entries {
        // SAFETY: Before callback registration; runtime caller owns machine setup.
        if let Err(error) = unsafe { session.spawn(entry) } {
            for slot in 0..session.tasks.len() {
                if session.tasks[slot].take().is_some() {
                    // SAFETY: These tasks have never run; all context users are gone.
                    unsafe {
                        stacks::release(frames, slot);
                    }
                }
            }
            assert_eq!(frames.allocated(), session.baseline);
            return Err(error);
        }
    }
    let session = Box::into_raw(session);
    ACTIVE.store(session, Ordering::Release);
    // SAFETY: Pin session until callback removal. Never retain a Rust reference
    // across suspend()/wait_for_ticks(), where the callback gains exclusive access.
    unsafe {
        arch::set_switch_handler(Some(schedule));
        loop {
            (*session).service();
            if (*session).scheduler.finished() {
                break;
            }
            arch::suspend();
            let idle = (*session).service.is_none()
                && !(*session).scheduler.finished()
                && !(*session)
                    .scheduler
                    .states()
                    .iter()
                    .any(|state| matches!(state, TaskState::Ready | TaskState::Exited));
            if idle {
                arch::wait_for_ticks(1);
            }
        }
        arch::set_switch_handler(None);
        ACTIVE.store(core::ptr::null_mut(), Ordering::Release);
        let session = Box::from_raw(session);
        assert_eq!(session.stats.exits, session.stats.spawned);
        assert_eq!(frames.allocated(), session.baseline);
        Ok(session.stats)
    }
}

unsafe fn schedule(context: &Context, cause: SwitchCause, now: u64) -> *const Context {
    // SAFETY: IRQs masked; run pins ACTIVE, and no API borrows across suspension.
    let session = unsafe { &mut *ACTIVE.load(Ordering::Acquire) };
    let previous = session.scheduler.current();
    if let Some(slot) = previous {
        session.tasks[slot].as_mut().unwrap().context = context.clone();
    } else {
        session.boot = context.clone();
    }
    let request = if cause == SwitchCause::Timer {
        Request::Schedule(Event::Timer)
    } else {
        core::mem::replace(&mut session.pending, Request::Schedule(Event::Yield))
    };
    let next = match request {
        Request::Spawn(entry) => {
            session.service = Some((previous.expect("spawn outside task"), entry));
            session.scheduler.park(Event::Yield, now);
            None
        }
        Request::Schedule(Event::Exit) => {
            session.stats.exits += 1;
            session.scheduler.park(Event::Exit, now);
            None
        }
        Request::Schedule(event) => session
            .scheduler
            .advance(event, now, session.config.preempt),
    };
    if previous != next {
        session.stats.switches += 1;
        if cause == SwitchCause::Timer
            && session.config.preempt
            && let Some(slot) = previous
        {
            session.stats.preemptions[slot] += 1;
        }
    }
    match next {
        Some(slot) => &session.tasks[slot].as_ref().unwrap().context,
        None => &session.boot,
    }
}

fn task_entry(slot: usize) -> ! {
    let entry = access(|session| session.tasks[slot].as_ref().unwrap().entry);
    entry();
    super::request(Request::Schedule(Event::Exit));
    panic!("exited task was resumed");
}

pub(super) fn access<R>(body: impl FnOnce(&mut Session) -> R) -> R {
    // SAFETY: Single kernel CPU. Closures are internal and cannot suspend or leak
    // references. IRQ masking protects mutable state from the timer callback.
    unsafe {
        arch::without_interrupts(|| {
            let session = ACTIVE.load(Ordering::Acquire);
            assert!(!session.is_null(), "task API outside session");
            body(&mut *session)
        })
    }
}
