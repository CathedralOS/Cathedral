//! Single-CPU task runner. Policy is scheduler.rs; architecture owns the saved
//! CPU image. IRQ callbacks never allocate, acquire locks, or retain references
//! into the shared interrupt stack. Task return drops locals before exit.

use crate::scheduler::{Event, Scheduler};
use alloc::boxed::Box;
use cathedral_arch::{self as arch, Context, StackRange, SwitchCause};
use core::sync::atomic::{AtomicPtr, Ordering};

#[derive(Clone, Copy, Debug, Default)]
pub struct RunStats {
    pub switches: u64,
    pub preemptions: [u64; 2],
    pub exits: usize,
}

struct Session {
    scheduler: Scheduler<2>,
    contexts: [Context; 2],
    boot: Context,
    entries: [fn(); 2],
    pending: Event,
    preempt: bool,
    stats: RunStats,
}

static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

/// Run two tasks to completion, returning on the caller's original stack.
/// Sleeping all tasks returns to the boot context, which waits for timer wakes.
/// # Safety
/// Sole CPU, IRQs off, initialized heap/timer/IDT. Each supplied stack is mapped,
/// exclusively owned, non-overlapping, and stays live until this call returns.
/// Tasks are trusted kernel functions; they must return to release their locals.
pub unsafe fn run(stacks: [StackRange; 2], entries: [fn(); 2], preempt: bool) -> RunStats {
    assert!(
        ACTIVE.load(Ordering::Acquire).is_null(),
        "nested task session"
    );
    // SAFETY: Caller provides the two unique stacks and single-CPU setup.
    let contexts = unsafe {
        [
            Context::new(stacks[0], task_entry, 0),
            Context::new(stacks[1], task_entry, 1),
        ]
    };
    let session = Box::into_raw(Box::new(Session {
        scheduler: Scheduler::new(),
        contexts,
        boot: Context::default(),
        entries,
        pending: Event::Yield,
        preempt,
        stats: RunStats::default(),
    }));
    ACTIVE.store(session, Ordering::Release);
    // SAFETY: The heap-owned session remains pinned until the callback is removed.
    // No Rust reference to it spans suspension or a period with IRQs enabled.
    unsafe {
        arch::set_switch_handler(Some(schedule));
        arch::suspend();
        while !(*session).scheduler.finished() {
            arch::wait_for_ticks(1);
        }
        arch::set_switch_handler(None);
        ACTIVE.store(core::ptr::null_mut(), Ordering::Release);
        let session = Box::from_raw(session);
        assert_eq!(session.stats.exits, 2);
        // Drop reclaims all context metadata. The reserved stack pool is reusable.
        session.stats
    }
}

unsafe fn schedule(context: &Context, cause: SwitchCause, now: u64) -> *const Context {
    // SAFETY: The installed callback has exclusive access with IRQs masked;
    // run pins ACTIVE and task APIs release all borrows before suspending.
    let session = unsafe { &mut *ACTIVE.load(Ordering::Acquire) };
    let previous = session.scheduler.current();
    if let Some(index) = previous {
        session.contexts[index] = context.clone();
    } else {
        session.boot = context.clone();
    }
    let event = if cause == SwitchCause::Timer {
        Event::Timer
    } else {
        core::mem::replace(&mut session.pending, Event::Yield)
    };
    if matches!(event, Event::Exit) {
        session.stats.exits += 1;
    }
    let next = session.scheduler.advance(event, now, session.preempt);
    if previous != next {
        session.stats.switches += 1;
        if cause == SwitchCause::Timer
            && session.preempt
            && let Some(index) = previous
        {
            session.stats.preemptions[index] += 1;
        }
    }
    match next {
        Some(index) => &session.contexts[index],
        None => &session.boot,
    }
}

fn task_entry(index: usize) -> ! {
    // SAFETY: Fetch only this immutable function pointer, with no session borrow
    // surviving IRQ re-enable or the task call.
    let entry =
        unsafe { arch::without_interrupts(|| (*ACTIVE.load(Ordering::Acquire)).entries[index]) };
    entry();
    request(Event::Exit);
    panic!("exited task was resumed");
}

fn request(event: Event) {
    // SAFETY: Kernel task context. Masking prevents a timer changing current
    // between publishing the request and taking vector 48. No borrow crosses it.
    unsafe {
        arch::without_interrupts(|| {
            let session = ACTIVE.load(Ordering::Acquire);
            assert!(!session.is_null(), "task operation outside a task session");
            assert!((*session).scheduler.current().is_some());
            (*session).pending = event;
            arch::suspend();
        });
    }
}

pub fn yield_now() {
    request(Event::Yield);
}

pub fn sleep(ticks: u64) {
    assert!(ticks < (1 << 63), "sleep exceeds clock comparison range");
    request(Event::Sleep(ticks));
}
