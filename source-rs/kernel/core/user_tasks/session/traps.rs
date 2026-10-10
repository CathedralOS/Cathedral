//! Interrupt/syscall entrance: save context, dispatch syscalls, wake waiters and select a root.
//! Allocating and reclaiming remain deferred to execution.rs on the boot stack.
use super::{ACTIVE, Exit, arch, keyboard, lifecycle, syscalls, taskcalls};
use crate::scheduler::{Event, TaskState};
use cathedral_arch::{Context, SwitchCause};
use core::sync::atomic::Ordering;
pub(super) unsafe fn schedule(context: &Context, cause: SwitchCause, now: u64) -> *const Context {
    // SAFETY: IRQs masked, kernel root active, run pins the session across syscalls.
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
                event = syscalls::syscall(session, slot, now);
            }
        }
    } else {
        assert!(!context.is_user());
        session.boot = context.clone();
    }
    if matches!(cause, SwitchCause::Timer | SwitchCause::Device) {
        keyboard::poll(session, now);
        syscalls::wake_receivers(session);
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
