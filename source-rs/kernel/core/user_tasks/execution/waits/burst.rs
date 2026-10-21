//! Real completion burst: mixed readiness/receives, deadline ties and boot-only drain.
use super::*;
use crate::user_tasks::Frames;
use crate::user_tasks::execution::deferred;
use crate::user_tasks::{
    Config, Executable, Program, address_space::USER_STACK, admission, startup,
};
use crate::{
    ipc::EndpointSpec,
    memory::FrameAllocator,
    scheduler::{Event, TaskState},
};
use cathedral_arch::{BootLayout, ImageRange};
use cathedral_kernel_contracts::user as abi;

const TASKS: usize = 25;

/// Exercise production completion and deferred routing against inactive user roots.
/// # Safety
/// Sole CPU, IRQs masked, initialized heap and exclusively owned allocator/layout.
/// No user context may run; output must be bounded and cannot suspend.
pub unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &BootLayout,
    image: ImageRange,
    output: fn(&[u8]) -> bool,
) {
    let baseline = frames.allocated();
    let endpoints: [_; TASKS - 1] = core::array::from_fn(|i| EndpointSpec {
        sender: 0,
        receiver: i + 1,
        revoker: Some(0),
    });
    let clock_readers: [_; TASKS] = core::array::from_fn(|i| i);
    let config = Config {
        executables: &[],
        dynamic_tasks: 0,
        frame_limit: usize::MAX,
        endpoints: &endpoints,
        links: &[],
        supervision: &[],
        channel_limits: &[],
        cpu_limits: &[],
        clock_readers: &clock_readers,
        memory: &[],
    };
    let (mut session, _) = startup::prepare(TASKS, &config, baseline, output).unwrap();
    let mut frames = Frames::new(frames, session.memory.funding.capacity(), usize::MAX).unwrap();
    let program = Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    };
    for slot in 0..TASKS {
        // SAFETY: Every prepared root remains inactive throughout this fixture.
        unsafe { admission::admit(&mut session, layout, image, &program, &mut frames, slot) }
            .unwrap();
    }
    let now = cathedral_arch::ticks();
    park_and_signal(&mut session, now);
    wake(&mut session, now);
    assert!(deferred::pending(&session));
    assert_eq!(ready(&session), COMPLETION_BATCH);
    assert!(session.tasks[1..17].iter().all(|t| t.receive.is_some()));
    assert!(session.tasks[17..].iter().all(|t| t.notify.is_none()));
    assert_eq!(session.tasks[24].report.rejected, 1);
    assert_eq!(session.tasks[24].report.wait_timeouts, 0); // Terminal wins expiry.
    let mut turns = 0;
    while deferred::pending(&session) {
        let before = ready(&session);
        // SAFETY: Same production step as an idle boot loop; no user root executes.
        unsafe { deferred::step(&mut session, &mut frames, layout, image, &config) };
        assert_eq!(ready(&session) - before, COMPLETION_BATCH);
        turns += 1;
        assert!(turns <= 2);
    }
    assert_eq!(turns, 2);
    // SAFETY: The fixture retains every inactive root and checked stack backing.
    unsafe { verify_completions(&session) };
    session.timing.report(output);
    for slot in 0..TASKS {
        // SAFETY: Inactive fixture roots have no page-object aliases or outstanding copies.
        unsafe { admission::retire(&mut session, slot, &mut frames) };
    }
    assert_eq!(frames.allocated(), baseline);
    assert_eq!(session.memory.funding.charged(), 0);
    output(b"wait completion: bounded mixed burst, deadline ties and boot-only drain passed\n");
}

fn park_and_signal(session: &mut Session, now: u64) {
    // Use an already elapsed interval so production boot steps can use the real
    // current tick without moving the registry clock backwards. Wrap is intentional.
    let registered = now.wrapping_sub(2);
    let deadline = now.wrapping_sub(1);
    for slot in 1..TASKS {
        let receive = session.ipc.handle(slot, 0).unwrap();
        let mut event = Event::Continue;
        if slot <= 16 {
            assert_eq!(
                syscalls::ipc::dispatch(
                    session,
                    slot,
                    if slot == 9 {
                        [abi::IPC_RECEIVE, receive, USER_STACK, 1]
                    } else {
                        [abi::IPC_RECEIVE_UNTIL, receive, USER_STACK, deadline]
                    },
                    registered,
                    &mut event
                ),
                Ok(0)
            );
        } else {
            assert_eq!(
                syscalls::ipc::readiness::syscall(
                    session,
                    slot,
                    syscalls::ipc::readiness::Wait {
                        handles: [receive; abi::MAX_WAIT_HANDLES],
                        kinds: [abi::WAIT_IPC; abi::MAX_WAIT_HANDLES],
                        count: 1,
                        deadline,
                    },
                    registered,
                    &mut event
                ),
                Ok(0)
            );
        }
        assert!(matches!(event, Event::Block));
    }
    // Park everyone: no runnable user and no fresh hardware event is needed to drain.
    for _ in 0..TASKS {
        assert!(
            session
                .scheduler
                .advance(Event::Yield, now, false)
                .is_some()
        );
        session.scheduler.park(Event::Block, now);
    }
    assert!(
        session
            .scheduler
            .states()
            .iter()
            .all(|s| *s == TaskState::Blocked)
    );
    for slot in (1..=9).chain(17..24) {
        let send = session.ipc.handle(0, ((slot - 1) * 2) as u64).unwrap();
        session.ipc.send(0, send, b"burst").unwrap();
    }
    let revoke = session.ipc.handle(0, 47).unwrap();
    session.ipc.revoke(0, revoke).unwrap();
    let revoke = session.ipc.handle(0, 19).unwrap();
    session.ipc.revoke(0, revoke).unwrap();
}

unsafe fn verify_completions(session: &Session) {
    for slot in 1..=8 {
        assert_eq!(session.tasks[slot].report.ipc_received, 1);
        assert_eq!(session.tasks[slot].report.wait_timeouts, 0); // Data wins expiry.
        let mut bytes = [0; 5];
        // SAFETY: Fixture retains the inactive root and its checked stack backing.
        assert!(unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_from_user(USER_STACK, &mut bytes)
        });
        assert_eq!(&bytes, b"burst");
    }
    let receive = session.ipc.handle(9, 0).unwrap();
    assert!(session.ipc.ready(9, receive).unwrap()); // Too-small destination preserves data.
    assert_eq!(session.tasks[9].report.rejected, 1);
    assert_eq!(session.tasks[9].report.ipc_received, 0);
    assert_eq!(session.tasks[10].report.rejected, 1);
    assert_eq!(session.tasks[10].report.wait_timeouts, 0); // Terminal receive wins expiry.
    for slot in 11..=16 {
        assert_eq!(session.tasks[slot].report.wait_timeouts, 1);
    }
    for slot in 17..24 {
        let receive = session.ipc.handle(slot, 0).unwrap();
        assert!(session.ipc.ready(slot, receive).unwrap()); // Observation did not consume.
        assert_eq!(session.tasks[slot].report.wait_timeouts, 0);
    }
}

fn ready(session: &Session) -> usize {
    session
        .scheduler
        .states()
        .iter()
        .filter(|&&state| state == TaskState::Ready)
        .count()
}
