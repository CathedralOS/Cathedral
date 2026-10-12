//! Pinned trusted-task state and IRQ-masked access; no borrow spans suspension.
use super::{Config, SpawnError};
use crate::{
    memory::FrameAllocator,
    scheduler::{Event, Scheduler, TaskId},
};
use alloc::{boxed::Box, vec::Vec};
use cathedral_arch::{self as arch, Context};
use core::sync::atomic::{AtomicPtr, Ordering};

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
    pub(super) context: Context,
    pub(super) entry: fn(),
    pub reply: Option<Result<TaskId, SpawnError>>,
}
pub(super) struct Session {
    pub scheduler: Scheduler,
    pub tasks: Vec<Option<Box<Task>>>,
    pub frames: *mut FrameAllocator,
    pub pending: Request,
    pub(super) boot: Context,
    pub(super) service: Option<(usize, fn())>,
    pub(super) config: Config,
    pub(super) baseline: usize,
    pub(super) stats: RunStats,
}
pub(super) static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

impl Session {
    pub(super) fn new(frames: &mut FrameAllocator, config: Config) -> Box<Self> {
        Box::new(Self {
            scheduler: Scheduler::new(config.task_limit),
            tasks: (0..config.task_limit).map(|_| None).collect(),
            frames,
            pending: Request::Schedule(Event::Yield),
            boot: Context::default(),
            service: None,
            config,
            baseline: frames.allocated(),
            stats: RunStats::default(),
        })
    }
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
