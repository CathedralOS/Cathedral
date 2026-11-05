//! Real inactive roots: cancellation submission cannot complete another wait.
use super::*;
use crate::memory::FrameAllocator;
use crate::scheduler::{Event, TaskState};
use crate::user_tasks::address_space::USER_STACK;

/// # Safety
/// Sole boot CPU, IRQs off, initialized heap and exclusive frame custody.
pub unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    output: fn(&[u8]) -> bool,
) {
    for blocking in [false, true] {
        // SAFETY: Fixture roots are never entered, and all backing stays local.
        unsafe { scenario(frames, layout, image, output, blocking) };
    }
    output(
        b"child cancellation: submit/poll, unrelated IPC wait and blocking compatibility passed\n",
    );
}

unsafe fn scenario(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    output: fn(&[u8]) -> bool,
    blocking: bool,
) {
    let baseline = frames.allocated();
    // SAFETY: Production admission creates two branches and their descendants.
    let (mut session, mut frames, ticket) =
        unsafe { super::withdrawal_test::prepare(frames, layout, image, output, true) };
    let mut event = Event::Continue;
    let now = arch::ticks();
    let poll = (
        abi::TASK_TRY_WAIT,
        ticket,
        USER_STACK,
        abi::EXIT_BYTES as u64,
    );
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, poll, &mut event, now),
        Err(abi::WOULD_BLOCK)
    );
    assert!(matches!(event, Event::Continue));
    assert!(session.tasks[0].wait.is_none() && !session.waits.pending());
    let begin = (abi::TASK_CANCEL_BEGIN, ticket, 0, 0);
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 1, begin, &mut event, now),
        Err(abi::BAD_HANDLE)
    );
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, (begin.0, ticket, 1, 0), &mut event, now),
        Err(abi::INVALID_ARGUMENT)
    );
    assert!(!session.launches[0].model.cancel_child());
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, begin, &mut event, now),
        Ok(0)
    );
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, begin, &mut event, now),
        Ok(0)
    );
    assert!(matches!(event, Event::Continue));
    assert_eq!(session.cancel_waiters, 0);

    assert_eq!(session.scheduler.advance(Event::Yield, now, false), Some(0));
    let [send, receive] = session.ipc.create(0).unwrap();
    if blocking {
        assert_eq!(
            syscalls::tasks::dispatch(
                &mut session,
                0,
                (abi::TASK_CANCEL, ticket, 0, 0),
                &mut event,
                now
            ),
            Ok(0)
        );
        assert_eq!(session.cancel_waiters, 1);
    } else {
        assert_eq!(
            syscalls::ipc::dispatch(
                &mut session,
                0,
                [
                    abi::IPC_RECEIVE,
                    receive,
                    USER_STACK,
                    abi::MAX_MESSAGE as u64
                ],
                now,
                &mut event
            ),
            Ok(0)
        );
    }
    assert!(matches!(event, Event::Block));
    session.scheduler.park(event, now);
    session.tasks[0].context.set_result(0xfeed);
    let untouched = session.tasks[0].context.syscall();
    let mut finished = session.tasks[0].context.clone();
    finished.set_result(0);
    let mut visits = 0;
    while session.deferred.reclaiming() {
        // SAFETY: All roots are inactive; production cleanup owns each alias.
        unsafe { reap(&mut session, &mut frames) };
        visits += 1;
        assert!(visits < abi::MAX_TASKS * 256);
        if !blocking {
            assert_eq!(session.scheduler.states()[0], TaskState::Blocked);
            assert_eq!(session.tasks[0].context.syscall(), untouched);
            assert!(session.tasks[0].receive.is_some());
        }
    }
    assert!(visits > 1);
    assert_eq!(session.cancel_waiters, 0);
    if blocking {
        assert_eq!(session.scheduler.states()[0], TaskState::Ready);
        assert_eq!(session.tasks[0].context.syscall(), finished.syscall());
    } else {
        // Only actual delivery may complete the parent's unrelated wait.
        session.ipc.send(0, send, b"ready").unwrap();
        crate::user_tasks::execution::waits::wake(&mut session, arch::ticks());
        assert_eq!(session.scheduler.states()[0], TaskState::Ready);
        assert_eq!(session.tasks[0].report.ipc_received, 1);
        assert!(session.tasks[0].receive.is_none());
    }
    event = Event::Continue;
    assert_eq!(
        syscalls::tasks::dispatch(
            &mut session,
            0,
            (poll.0, ticket, 0, poll.3),
            &mut event,
            now
        ),
        Err(abi::BAD_ADDRESS)
    );
    assert!(session.launches[0].model.wait(0, ticket).unwrap().is_some());
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, begin, &mut event, now),
        Ok(0)
    );
    assert_eq!(session.cancel_waiters, 0);
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, poll, &mut event, now),
        Ok(0)
    );
    let mut record = [0; abi::EXIT_BYTES];
    // SAFETY: Completed copyout in this inactive root's checked stack allocation.
    assert!(unsafe {
        session.tasks[0]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(USER_STACK, &mut record)
    });
    assert_eq!(
        u64::from_le_bytes(record[..8].try_into().unwrap()),
        abi::EXIT_CANCELLED
    );
    assert!(record[8..].iter().all(|byte| *byte == 0));
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, poll, &mut event, now),
        Err(abi::BAD_HANDLE)
    );
    assert_eq!(
        syscalls::tasks::dispatch(&mut session, 0, begin, &mut event, now),
        Err(abi::BAD_HANDLE)
    );
    assert!(matches!(event, Event::Continue));

    session.scheduler.cancel(0);
    session.tasks[0].report.exit = Some(Exit::Cancelled);
    close(&mut session, 0);
    while session.deferred.reclaiming() {
        // SAFETY: Owner death withdraws the remaining branch before retiring it.
        unsafe { reap(&mut session, &mut frames) };
        visits += 1;
        assert!(visits < abi::MAX_TASKS * 512);
    }
    assert!(!session.waits.pending());
    // SAFETY: Sole surviving inactive root, without any outstanding receive.
    unsafe { admission::retire(&mut session, 1, &mut frames) };
    assert_eq!(frames.allocated(), baseline);
    assert_eq!(session.memory.funding.charged(), 0);
}
