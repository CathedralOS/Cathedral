//! Admission/reclamation happen on the boot stack; callbacks never allocate.

use super::{Error, Program};
use crate::ipc::{EndpointSpec, Ipc, MAX_EPOCH};
mod admission;
mod dispatch;
use crate::{
    extent::FrameAllocator,
    scheduler::{Event, Scheduler, TaskState},
};
use alloc::vec::Vec;
use cathedral_arch::{self as arch, Context, StackFrames, SwitchCause, UserSpace};
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Returned(u64),
    Fault(arch::UserFault),
}
#[derive(Default, Debug)]
pub struct Report {
    pub exit: Option<Exit>,
    pub preemptions: usize,
    pub yields: usize,
    pub writes: usize,
    pub rejected: usize,
    pub completion_order: usize,
    pub frames: usize,
    pub receives_blocked: usize,
    pub ipc_sent: usize,
    pub ipc_received: usize,
}
struct Task {
    context: Context,
    space: Option<UserSpace>,
    report: Report,
    receive: Option<dispatch::Receive>,
}
struct Session {
    tasks: Vec<Task>,
    scheduler: Scheduler,
    boot: Context,
    output: fn(&[u8]) -> bool,
    completed: usize,
    ipc: Ipc,
}
static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

static EPOCH: AtomicU64 = AtomicU64::new(1);

pub struct Config<'a> {
    pub frame_limit: usize,
    pub endpoints: &'a [EndpointSpec],
}

struct Frames<'a> {
    frames: &'a mut FrameAllocator,
    remaining: usize,
}
// SAFETY: Run is exclusive on the boot CPU, with the kernel's identity map active.
unsafe impl StackFrames for Frames<'_> {
    fn allocate(&mut self) -> Option<u64> {
        if self.remaining == 0 {
            return None;
        }
        let frame = self.frames.allocate()?;
        self.remaining -= 1;
        Some(frame.address())
    }
    unsafe fn release(&mut self, address: u64) {
        self.frames
            .release_address(address)
            .expect("invalid user frame release");
    }
}

/// Run a bounded session, returning task outcomes only after complete reclamation.
/// frame_limit supports deterministic admission-failure testing.
/// # Safety
/// Sole CPU, IRQs off, no kernel/user task session; initialized heap/IDT/timer.
/// Kernel root, layout and image are live; frame reclamation is enabled. Output
/// is bounded, cannot allocate/suspend, and does not retain its borrowed buffer.
pub unsafe fn run(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    programs: &[Program<'_>],
    output: fn(&[u8]) -> bool,
    frame_limit: usize,
) -> Result<Vec<Report>, Error> {
    // SAFETY: Same session and memory obligations as the caller.
    unsafe {
        run_configured(
            frames,
            layout,
            image,
            programs,
            output,
            Config {
                frame_limit,
                endpoints: &[],
            },
        )
    }
}

/// Run tasks with explicit boot-issued endpoint grants.
/// # Safety
/// Same obligations as run. Endpoint task indices refer to this exact program list.
pub unsafe fn run_configured(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    programs: &[Program<'_>],
    output: fn(&[u8]) -> bool,
    config: Config<'_>,
) -> Result<Vec<Report>, Error> {
    assert!(ACTIVE.load(Ordering::Acquire).is_null());
    if programs.is_empty() || programs.len() > 8 {
        return Err(Error::InvalidCount);
    }
    let epoch = EPOCH
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            (value <= MAX_EPOCH).then_some(value + 1)
        })
        .map_err(|_| Error::InvalidEndpoints)?;
    let ipc =
        Ipc::new(epoch, programs.len(), config.endpoints).map_err(|_| Error::InvalidEndpoints)?;
    let baseline = frames.allocated();
    let mut session = Session {
        tasks: Vec::new(),
        scheduler: Scheduler::new(programs.len()),
        boot: Context::default(),
        output,
        completed: 0,
        ipc,
    };
    session
        .tasks
        .try_reserve_exact(programs.len())
        .map_err(|_| Error::OutOfHeap)?;
    let mut reports = Vec::new();
    reports
        .try_reserve_exact(programs.len())
        .map_err(|_| Error::OutOfHeap)?;
    let mut source = Frames {
        frames,
        remaining: config.frame_limit,
    };
    for (slot, program) in programs.iter().enumerate() {
        // Keep large address-space construction temporaries out of the session frame.
        if let Err(error) =
            // SAFETY: Serialized admission on kernel CR3; no user has started.
            unsafe {
                admission::admit(&mut session, layout, image, program, &mut source, slot)
            }
        {
            for task in &mut session.tasks {
                // SAFETY: Admission failed before publishing any context/root.
                unsafe {
                    admission::retire(task, &mut source);
                }
            }
            assert_eq!(source.frames.allocated(), baseline);
            return Err(error);
        }
    }
    let session = &raw mut session;
    ACTIVE.store(session, Ordering::Release);
    // SAFETY: Session remains pinned on mapped boot stack. Its vectors never grow;
    // callbacks get exclusive access only across suspension, never a live borrow.
    unsafe {
        arch::begin_user_session();
        arch::set_switch_handler(Some(schedule));
        loop {
            for slot in 0..(*session).tasks.len() {
                if (*session).scheduler.states()[slot] == TaskState::Exited {
                    admission::retire(&mut (&mut (*session).tasks)[slot], &mut source);
                    (*session).scheduler.reap(slot);
                }
            }
            if (*session).scheduler.finished() {
                break;
            }
            if (*session).scheduler.states().contains(&TaskState::Ready) {
                arch::suspend();
            } else {
                arch::wait_for_ticks(1);
            }
        }
        arch::set_switch_handler(None);
        arch::end_user_session();
        ACTIVE.store(core::ptr::null_mut(), Ordering::Release);
        for task in &mut (*session).tasks {
            reports.push(core::mem::take(&mut task.report));
        }
    }
    assert_eq!(source.frames.allocated(), baseline);
    Ok(reports)
}

unsafe fn schedule(context: &Context, cause: SwitchCause, now: u64) -> *const Context {
    // SAFETY: IRQs masked, kernel root active, run pins the session across dispatch.
    let session = unsafe { &mut *ACTIVE.load(Ordering::Acquire) };
    let previous = session.scheduler.current();
    let mut event = if cause == SwitchCause::Timer {
        Event::Timer
    } else {
        Event::Yield
    };
    if let Some(slot) = previous {
        assert!(context.is_user());
        let task = &mut session.tasks[slot];
        task.context = context.clone();
        match cause {
            SwitchCause::Timer => {}
            SwitchCause::Yield => panic!("user entered privileged yield gate"),
            SwitchCause::Fault(fault) => {
                task.report.exit = Some(Exit::Fault(fault));
                event = Event::Exit;
            }
            SwitchCause::Syscall => {
                event = dispatch::syscall(session, slot);
            }
        }
    } else {
        assert!(!context.is_user());
        session.boot = context.clone();
    }
    let next = if matches!(event, Event::Exit) {
        session.ipc.close_task(previous.unwrap());
        dispatch::wake_receivers(session);
        session.completed += 1;
        session.tasks[previous.unwrap()].report.completion_order = session.completed;
        session.scheduler.park(event, now);
        None // Always reclaim from the boot context, never an interrupt stack.
    } else {
        session.scheduler.advance(event, now, true)
    };
    if cause == SwitchCause::Timer
        && previous != next
        && let Some(slot) = previous
    {
        session.tasks[slot].report.preemptions += 1;
    }
    // SAFETY: Selected context and root stay live until the next trap; boot uses
    // its original CR3. Assembly restores only after this Rust borrow has ended.
    unsafe {
        arch::select_user_root(next.map(|slot| session.tasks[slot].space.as_ref().unwrap().root()));
    }
    next.map_or(&session.boot, |slot| &session.tasks[slot].context)
}
