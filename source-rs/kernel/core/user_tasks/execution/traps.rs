//! Interrupt/syscall entrance: save context, dispatch syscalls, wake waiters and select a root.
//! Allocating and reclaiming remain deferred to execution/execution.rs on the boot stack.
use crate::scheduler::Event;
use crate::user_tasks::execution::timing::{self, Phase as TimedPhase};
use crate::user_tasks::{ACTIVE, Exit, arch, keyboard, lifecycle, syscalls};
use cathedral_arch::{Context, SwitchCause};
use core::sync::atomic::Ordering;
pub(super) unsafe fn schedule(context: &Context, cause: SwitchCause, now: u64) -> *const Context {
    // SAFETY: IRQs masked, kernel root active, run pins the session across syscalls.
    let session = unsafe { &mut *ACTIVE.load(Ordering::Acquire) };
    let trap_start = timing::start();
    let previous = session.scheduler.current();
    let mut event = match cause {
        SwitchCause::Timer => Event::Timer,
        // A device IRQ records readiness; it is not a fresh round-robin quantum.
        // Runnable waiters get their turn on blocking/yield or the next timer.
        SwitchCause::Device => Event::Continue,
        _ => Event::Yield,
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
                let start = timing::start();
                event = syscalls::syscall(session, slot, now);
                session.timing.record(TimedPhase::Syscall, start);
            }
        }
    } else {
        assert!(!context.is_user());
        session.boot = context.clone();
    }
    if matches!(cause, SwitchCause::Timer | SwitchCause::Device) {
        let start = timing::start();
        syscalls::devices::leased::interrupts(session);
        keyboard::poll(session, now);
        super::waits::wake(session, now);
        session.timing.record(TimedPhase::InterruptWork, start);
    }
    let next = if matches!(event, Event::Exit) {
        lifecycle::close(session, previous.unwrap());
        session.completed += 1;
        session.tasks[previous.unwrap()].report.completion_order = session.completed;
        session.scheduler.account_cpu(arch::cpu_time_us());
        session.scheduler.park(event, now);
        None // Always reclaim from the boot context, never an interrupt stack.
    } else if super::deferred::return_to_boot(
        super::deferred::pending(session),
        previous.is_some(),
        cause == SwitchCause::Yield,
    ) {
        session.scheduler.account_cpu(arch::cpu_time_us());
        session.scheduler.park(event, now);
        // Async boot IRQs return to boot so peers cannot starve collection.
        // Only an explicit yield after a coherent step may dispatch a peer;
        // that peer's next trap returns here while work remains.
        None // Admission/reclamation use the boot stack, never the trap stack.
    } else {
        session.scheduler.account_cpu(arch::cpu_time_us());
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
    session.timing.record(TimedPhase::Trap, trap_start);
    #[cfg(feature = "memory-lab")]
    session.timing.wait_selected(next);
    next.map_or(&session.boot, |slot| &session.tasks[slot].context)
}
