//! Allocation-free context selection; spawn and exit defer lifetime work to the boot stack.
use super::{ACTIVE, Event, Ordering, Request};
use cathedral_arch::{Context, SwitchCause};

pub(super) unsafe fn schedule(context: &Context, cause: SwitchCause, now: u64) -> *const Context {
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
