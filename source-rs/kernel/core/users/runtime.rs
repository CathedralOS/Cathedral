//! Admission/reclamation happen on the boot stack; callbacks never allocate.

use super::{Error, Executable, Program, elf, syscall};
use crate::{
    extent::FrameAllocator,
    scheduler::{Event, Scheduler, TaskState},
};
use alloc::vec::Vec;
use cathedral_arch::{self as arch, Context, StackFrames, SwitchCause, UserSpace};
use core::sync::atomic::{AtomicPtr, Ordering};

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
}
struct Task {
    context: Context,
    space: Option<UserSpace>,
    report: Report,
}
struct Session {
    tasks: Vec<Task>,
    scheduler: Scheduler,
    boot: Context,
    output: fn(&[u8]) -> bool,
    completed: usize,
}
static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

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
    assert!(ACTIVE.load(Ordering::Acquire).is_null());
    if programs.is_empty() || programs.len() > 8 {
        return Err(Error::InvalidCount);
    }
    let baseline = frames.allocated();
    let mut session = Session {
        tasks: Vec::new(),
        scheduler: Scheduler::new(programs.len()),
        boot: Context::default(),
        output,
        completed: 0,
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
        remaining: frame_limit,
    };
    for (slot, program) in programs.iter().enumerate() {
        // Parse before physical admission. Any error also retires earlier tasks.
        let admission = (|| {
            // SAFETY: Serialized admission on kernel CR3; no task has started.
            unsafe {
                match &program.executable {
                    Executable::Probe(code) => UserSpace::create(layout, image, code, &mut source)
                        .map(|space| (space, arch::USER_CODE))
                        .map_err(Error::Memory),
                    Executable::Elf(bytes) => {
                        let executable = elf::parse(bytes).map_err(Error::Executable)?;
                        UserSpace::from_segments(layout, image, executable.segments(), &mut source)
                            .map(|space| (space, executable.entry))
                            .map_err(Error::Memory)
                    }
                }
            }
        })();
        let (space, entry) = match admission {
            Ok(admitted) => admitted,
            Err(error) => {
                for task in &mut session.tasks {
                    // SAFETY: Admission failed before publishing any context/root.
                    unsafe {
                        task.space.take().unwrap().release(&mut source);
                    }
                }
                assert_eq!(source.frames.allocated(), baseline);
                return Err(error);
            }
        };
        let count = space.frame_count();
        session.tasks.push(Task {
            context: Context::user(entry, arch::USER_STACK_TOP, program.arguments),
            space: Some(space),
            report: Report {
                frames: count,
                ..Default::default()
            },
        });
        session.scheduler.admit(slot);
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
                    (&mut (*session).tasks)[slot]
                        .space
                        .take()
                        .unwrap()
                        .release(&mut source);
                    (*session).scheduler.reap(slot);
                }
            }
            if (*session).scheduler.finished() {
                break;
            }
            arch::suspend();
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
                let (number, first, second) = context.syscall();
                let result = match number {
                    syscall::WRITE => {
                        let mut buffer = [0u8; syscall::MAX_WRITE];
                        if second > syscall::MAX_WRITE as u64 {
                            syscall::INVALID_ARGUMENT
                        }
                        // SAFETY: Copy checks the entire range against this task's owned pages.
                        else if !unsafe {
                            task.space
                                .as_ref()
                                .unwrap()
                                .copy_from_user(first, &mut buffer[..second as usize])
                        } {
                            syscall::BAD_ADDRESS
                        } else {
                            if (session.output)(&buffer[..second as usize]) {
                                task.report.writes += 1;
                                second
                            } else {
                                syscall::IO_ERROR
                            }
                        }
                    }
                    syscall::YIELD => {
                        task.report.yields += 1;
                        0
                    }
                    syscall::EXIT => {
                        task.report.exit = Some(Exit::Returned(first));
                        event = Event::Exit;
                        0
                    }
                    _ => syscall::UNKNOWN,
                };
                if (result as i64) < 0 {
                    task.report.rejected += 1;
                }
                task.context.set_result(result);
            }
        }
    } else {
        assert!(!context.is_user());
        session.boot = context.clone();
    }
    let next = if matches!(event, Event::Exit) {
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
