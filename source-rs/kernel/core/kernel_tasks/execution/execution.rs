// Pin state and install trap dispatch, service deferred work, then unpublish before returning.
use super::session::{ACTIVE, Request, Session, access};
use crate::scheduler::{Event, TaskState};
use alloc::boxed::Box;
use cathedral_arch as arch;
use core::sync::atomic::Ordering;
use traps::schedule;

pub(super) unsafe fn run(session: Box<Session>) -> Box<Session> {
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
        Box::from_raw(session)
    }
}

pub(super) fn task_entry(slot: usize) -> ! {
    let entry = access(|session| session.tasks[slot].as_ref().unwrap().entry);
    entry();
    super::request(Request::Schedule(Event::Exit));
    panic!("exited task was resumed");
}
