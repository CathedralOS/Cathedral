//! Admission/reclamation happen on the boot stack; callbacks never allocate.

use super::{Error, Program};
use crate::ipc::{EndpointSpec, Ipc, MAX_EPOCH};
mod admission;
mod dispatch;
mod keyboard;
mod lifecycle;
mod links;
mod notify;
mod taskcalls;
use crate::supervision::Supervisor;
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
    Cancelled,
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
    pub readiness_blocked: usize,
    pub keyboard_reads_blocked: usize,
    pub ipc_sent: usize,
    pub ipc_received: usize,
    pub spawned: usize,
    pub reaped: usize,
    pub waits_blocked: usize,
    pub cancelled: usize,
    pub wait_timeouts: usize,
    pub cancelled_ready: usize,
    pub cancelled_blocked: usize,
}
struct Task {
    context: Context,
    space: Option<UserSpace>,
    report: Report,
    receive: Option<dispatch::Receive>,
    wait: Option<taskcalls::Wait>,
    notify: Option<notify::Wait>,
    keyboard_wait: bool,
    keyboard_deadline: Option<u64>,
}
struct Session {
    tasks: Vec<Task>,
    scheduler: Scheduler,
    boot: Context,
    output: fn(&[u8]) -> bool,
    completed: usize,
    ipc: Ipc,
    launches: Vec<LaunchState>,
    links: Vec<LinkState>,
    keyboard: crate::byte_queue::ByteQueue,
    frame_baseline: usize,
    clock: crate::deadline::Clock,
}
static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

static EPOCH: AtomicU64 = AtomicU64::new(1);

pub struct Config<'a> {
    pub frame_limit: usize,
    pub links: &'a [crate::link::LinkSpec],
    pub endpoints: &'a [EndpointSpec],
    pub supervision: &'a [Supervision<'a>],
    pub clock_readers: &'a [usize],
}

pub struct Supervision<'a> {
    /// Exclusive PC bootstrap byte channel; configuration/decoding stay in userspace.
    pub keyboard: bool,
    /// Exclusive boot-approved display aperture, granted only to the child.
    pub framebuffer: Option<cathedral_contracts::display::Framebuffer>,
    pub owner: usize,
    pub peer: usize,
    /// Boot selects executable and first entry argument; spawn supplies the second.
    pub program: Program<'a>,
    /// Physical admission budget per spawn, including deterministic failure probes.
    pub frame_limit: usize,
}

struct LaunchState {
    model: Supervisor,
    endpoint_base: usize,
    framebuffer: Option<cathedral_contracts::display::Framebuffer>,
    keyboard: bool,
}
struct LinkState {
    model: crate::link::Link,
    endpoint_base: usize,
}
const MAX_LAUNCHES: usize = 3;

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
                links: &[],
                frame_limit,
                endpoints: &[],
                supervision: &[],
                clock_readers: &[],
            },
        )
    }
}

/// Run tasks with explicit boot-issued endpoint, launch and clock grants.
/// # Safety
/// Same obligations as run. Endpoint indices refer to initial programs; clock
/// indices may also name reserved child slots.
/// Any framebuffer must be a live, reserved device aperture exclusively held by
/// boot: no RAM allocation or other CPU/alias may access its pages. The launch
/// grant authorizes its executable to read/write the entire aperture for its lifetime.
/// A keyboard grant requires exclusive PC-controller/PIC IRQ1 custody. This
/// bootstrap profile has no concurrent firmware, mouse or other controller user.
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
    let epoch = next_epoch().ok_or(Error::InvalidEndpoints)?;
    let count = programs.len() + config.supervision.len();
    if count > crate::ipc::MAX_TASKS || config.supervision.len() > MAX_LAUNCHES {
        return Err(Error::InvalidCount);
    }
    if config.endpoints.len() + 2 * (config.supervision.len() + config.links.len())
        > crate::ipc::MAX_ENDPOINTS
        || config
            .supervision
            .iter()
            .filter(|grant| grant.framebuffer.is_some())
            .count()
            > 1
        || config
            .supervision
            .iter()
            .filter(|grant| grant.keyboard)
            .count()
            > 1
    {
        return Err(Error::InvalidEndpoints);
    }
    let clock = crate::deadline::Clock::new(epoch, count, config.clock_readers)
        .map_err(|_| Error::InvalidEndpoints)?;
    let mut launches = Vec::new();
    launches
        .try_reserve_exact(config.supervision.len())
        .map_err(|_| Error::OutOfHeap)?;
    for (index, launch) in config.supervision.iter().enumerate() {
        if launch.framebuffer.is_some_and(|fb| !fb.valid())
            || launch.owner >= programs.len()
            || launch.peer >= programs.len()
        {
            return Err(Error::InvalidEndpoints);
        }
        let grant_epoch = next_epoch().ok_or(Error::InvalidEndpoints)?;
        launches.push(LaunchState {
            model: Supervisor::new(
                grant_epoch,
                launch.owner,
                launch.peer,
                programs.len() + index,
            )
            .map_err(|_| Error::InvalidEndpoints)?,
            endpoint_base: config.endpoints.len() + 2 * index,
            framebuffer: launch.framebuffer,
            keyboard: launch.keyboard,
        });
    }
    // Static grants may refer only to initial principals, never the reusable child slot.
    if config.endpoints.iter().any(|spec| {
        spec.sender >= programs.len()
            || spec.receiver >= programs.len()
            || spec.revoker.is_some_and(|slot| slot >= programs.len())
    }) {
        return Err(Error::InvalidEndpoints);
    }
    let mut links = Vec::new();
    links
        .try_reserve_exact(config.links.len())
        .map_err(|_| Error::OutOfHeap)?;
    for (index, &spec) in config.links.iter().enumerate() {
        // Links connect reserved children only in this bounded launch graph.
        if spec.client < programs.len() || spec.service < programs.len() {
            return Err(Error::InvalidEndpoints);
        }
        links.push(LinkState {
            model: crate::link::Link::new(
                next_epoch().ok_or(Error::InvalidEndpoints)?,
                spec,
                count,
                programs.len(),
            )
            .map_err(|_| Error::InvalidEndpoints)?,
            endpoint_base: config.endpoints.len() + 2 * (config.supervision.len() + index),
        });
    }
    let mut ipc = Ipc::new(epoch, count, config.endpoints).map_err(|_| Error::InvalidEndpoints)?;
    for launch in &launches {
        ipc.close_task(launch.model.child);
    }
    let baseline = frames.allocated();
    let mut session = Session {
        tasks: Vec::new(),
        scheduler: Scheduler::new(count),
        boot: Context::default(),
        output,
        completed: 0,
        ipc,
        launches,
        links,
        keyboard: crate::byte_queue::ByteQueue::new(),
        frame_baseline: baseline,
        clock,
    };
    session
        .tasks
        .try_reserve_exact(count)
        .map_err(|_| Error::OutOfHeap)?;
    let mut reports = Vec::new();
    reports
        .try_reserve_exact(count)
        .map_err(|_| Error::OutOfHeap)?;
    for _ in 0..count {
        admission::reserve_slot(&mut session);
    }
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
            for task in session.tasks.iter_mut().filter(|task| task.space.is_some()) {
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
        arch::keyboard_irq(config.supervision.iter().any(|grant| grant.keyboard));
        loop {
            lifecycle::reap(&mut *session, &mut source);
            for (index, launch) in config.supervision.iter().enumerate() {
                lifecycle::spawn(&mut *session, &mut source, layout, image, launch, index);
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
        arch::keyboard_irq(false);
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
            SwitchCause::Timer | SwitchCause::Device => {}
            SwitchCause::Yield => panic!("user entered privileged yield gate"),
            SwitchCause::Fault(fault) => {
                task.report.exit = Some(Exit::Fault(fault));
                event = Event::Exit;
            }
            SwitchCause::Syscall => {
                event = dispatch::syscall(session, slot, now);
            }
        }
    } else {
        assert!(!context.is_user());
        session.boot = context.clone();
    }
    if matches!(cause, SwitchCause::Timer | SwitchCause::Device) {
        keyboard::poll(session, now);
        dispatch::wake_receivers(session);
        taskcalls::wake(session, now);
    }
    let next = if matches!(event, Event::Exit) {
        lifecycle::close(session, previous.unwrap());
        session.completed += 1;
        session.tasks[previous.unwrap()].report.completion_order = session.completed;
        session.scheduler.park(event, now);
        None // Always reclaim from the boot context, never an interrupt stack.
    } else if session.scheduler.states().contains(&TaskState::Exited)
        || session
            .launches
            .iter()
            .any(|launch| launch.model.pending().is_some() || launch.model.cancellation_pending())
    {
        session.scheduler.park(event, now);
        // An IRQ can interrupt the restored idle boot context before it reaches
        // reap. Keep returning there until all deferred work is done; otherwise
        // runnable peers can indefinitely starve collection of an exited child.
        None // Admission/reclamation use the boot stack, never the trap stack.
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

fn next_epoch() -> Option<u64> {
    EPOCH
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            (value <= MAX_EPOCH).then_some(value + 1)
        })
        .ok()
}
