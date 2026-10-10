//! Trusted kernel task API. Admission and teardown run on the boot stack;
//! the interrupt callback only saves contexts and chooses where to resume.

mod runtime;
mod stacks;

pub use crate::scheduler::TaskId;
use crate::{extent::FrameAllocator, scheduler::Event};
use cathedral_arch as arch;
pub use runtime::RunStats;
use runtime::{Request, access};

#[derive(Clone, Copy)]
pub struct Config {
    pub task_limit: usize,
    pub preempt: bool,
    /// Bounds simultaneous stack backing plus paging-structure frames.
    pub frame_limit: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            task_limit: 8,
            preempt: true,
            frame_limit: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnError {
    InvalidLimit,
    TaskLimit,
    OutOfFrames,
    OutOfHeap,
}

/// Run a bounded task arena to completion, returning on the original stack.
/// IDs are scoped to this session. Each admitted task owns a fresh guarded stack.
/// # Safety
/// Sole CPU, IRQs off, initialized heap/timer/IDT and owned active page tables.
/// Frames have reclamation enabled and unique custody of identity-mapped RAM.
/// Trusted task functions must return normally to release Rust locals; no unwind.
pub unsafe fn run(
    frames: &mut FrameAllocator,
    config: Config,
    entries: &[fn()],
) -> Result<RunStats, SpawnError> {
    // SAFETY: Forward the runtime's machine/lifetime preconditions from caller.
    unsafe { runtime::run(frames, config, entries) }
}

/// Spawn from a running task. Returns only after admission or complete rollback.
pub fn spawn(entry: fn()) -> Result<TaskId, SpawnError> {
    request(Request::Spawn(entry));
    access(|session| {
        let slot = session.scheduler.current().expect("spawn outside task");
        session.tasks[slot]
            .as_mut()
            .unwrap()
            .reply
            .take()
            .expect("missing spawn reply")
    })
}

pub fn current_id() -> TaskId {
    access(|session| {
        session
            .scheduler
            .id(session.scheduler.current().expect("not in task"))
    })
}

pub fn is_alive(id: TaskId) -> bool {
    access(|session| session.scheduler.is_alive(id))
}

pub fn current_stack() -> arch::StackRange {
    arch::task_stack_range(current_id().slot())
}

/// Stable accounting snapshot; useful for admission policy and lab assertions.
pub fn allocated_frames() -> usize {
    access(|session| {
        // SAFETY: The session borrows this allocator for its entire run, IRQs off.
        unsafe { (*session.frames).allocated() }
    })
}

pub fn yield_now() {
    request(Request::Schedule(Event::Yield));
}
pub fn sleep(ticks: u64) {
    assert!(ticks < (1 << 63), "sleep exceeds clock comparison range");
    request(Request::Schedule(Event::Sleep(ticks)));
}

fn request(request: Request) {
    // SAFETY: Sole kernel CPU. No borrow or lock spans the suspension. The saved
    // context keeps IF clear until this function restores the caller's IF.
    unsafe {
        arch::without_interrupts(|| {
            access(|session| {
                assert!(
                    session.scheduler.current().is_some(),
                    "operation outside task"
                );
                session.pending = request;
            });
            arch::suspend();
        });
    }
}
