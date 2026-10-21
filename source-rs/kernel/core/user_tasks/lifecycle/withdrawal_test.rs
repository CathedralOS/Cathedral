//! Withdraw a lent subtree before retrying any descendant's queued receive.
use super::*;
use crate::user_tasks::execution::timing::Phase;
use crate::user_tasks::{
    Config, Executable, Program, Supervision, address_space::USER_STACK, startup,
};
use crate::{
    memory::FrameAllocator,
    scheduler::{Event, TaskState},
};

const TASKS: usize = abi::MAX_TASKS;
const LAUNCHES: usize = TASKS - 2; // Initial owner and unrelated survivor.
const GROUP: usize = abi::MAX_SPAWN_LOANS + 1;

/// # Safety
/// Sole boot CPU, IRQs masked, live kernel layout, initialized heap and exclusive
/// frame custody. Fixture roots never execute; output cannot suspend.
pub unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    output: fn(&[u8]) -> bool,
) {
    let baseline = frames.allocated();
    // SAFETY: Fresh inactive roots and production admission/loan commits.
    let (mut session, mut frames) = unsafe { prepare(frames, layout, image, output) };
    let [send, receive] = session.ipc.create(1).unwrap();
    receive_on(&mut session, 1, receive);
    for child in 2..TASKS {
        if session.launches[child - 2].model.owner != 0 {
            let receive = session.ipc.handle(child, 0).unwrap();
            receive_on(&mut session, child, receive);
        }
    }
    for _ in 0..TASKS {
        assert!(
            session
                .scheduler
                .advance(Event::Yield, arch::ticks(), false)
                .is_some()
        );
        session.scheduler.park(Event::Block, arch::ticks());
    }
    session.scheduler.cancel(0);
    session.tasks[0].report.exit = Some(Exit::Cancelled);
    close(&mut session, 0);
    // The owner is gone; withdrawal of its child and the lent grandchildren is
    // pending. Queue data before that sweep, without executing any user context.
    session.ipc.send(1, send, b"survivor").unwrap();
    for index in 0..LAUNCHES {
        let owner = session.launches[index].model.owner;
        if owner == 0 {
            continue;
        }
        let base = session.launches[index].endpoint_base;
        let local = session.ipc.accept_child(base, owner);
        let send = session.ipc.handle(owner, local).unwrap();
        session.ipc.send(owner, send, b"must not copy").unwrap();
    }
    let wakes = session.timing.count(Phase::WaitCompletion);
    let before = frames.allocated();
    // SAFETY: This production visit must withdraw all descendants before returning.
    unsafe { reap(&mut session, &mut frames) };
    assert_eq!(session.timing.count(Phase::WaitCompletion) - wakes, 1);
    assert_eq!(session.timing.count(Phase::Withdraw), 1);
    assert_eq!(session.timing.count(Phase::Close), 1); // Only the initial task close.
    assert!(!session.deferred.has(Work::Cancel));
    assert!(before - frames.allocated() <= reclamation::FRAME_BATCH);
    for child in 2..TASKS {
        assert_eq!(session.scheduler.states()[child], TaskState::Exited);
        assert!(session.tasks[child].space.is_some());
        assert_eq!(session.tasks[child].report.ipc_received, 0);
        assert_eq!(session.tasks[child].report.rejected, 0);
        assert_eq!(session.tasks[child].report.reaped, 0);
        assert!(session.ipc.handle(child, 0).is_err());
        assert!(session.tasks[child].receive.is_none());
    }
    for branch in (0..LAUNCHES).step_by(GROUP) {
        assert_eq!(
            session.tasks[branch + 2].report.cancelled_blocked,
            (LAUNCHES - branch - 1).min(abi::MAX_SPAWN_LOANS)
        );
    }
    assert_eq!(session.scheduler.states()[1], TaskState::Ready);
    assert_eq!(session.tasks[1].report.ipc_received, 1);
    let mut bytes = [0; 8];
    // SAFETY: The survivor's inactive root retains checked writable stack backing.
    assert!(unsafe {
        session.tasks[1]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(USER_STACK, &mut bytes)
    });
    assert_eq!(&bytes, b"survivor");
    let mut steps = 0;
    while session.deferred.reclaiming() {
        // SAFETY: All canceled roots are permanently inactive; no aliases escape.
        unsafe { reap(&mut session, &mut frames) };
        steps += 1;
        // Cover each task's bounded metadata scans and frame-retirement visits.
        assert!(steps < TASKS * 256);
        assert_eq!(session.scheduler.states()[1], TaskState::Ready);
    }
    assert!(steps > TASKS);
    assert!(
        session
            .tasks
            .iter()
            .enumerate()
            .all(|(slot, task)| slot == 1 || task.space.is_none())
    );
    assert!(!session.waits.pending());
    session.timing.report(output);
    // SAFETY: The sole remaining inactive root has no outstanding receive/copy.
    unsafe { admission::retire(&mut session, 1, &mut frames) };
    assert_eq!(frames.allocated(), baseline);
    assert_eq!(session.memory.funding.charged(), 0);
    output(b"subtree withdrawal: one retry batch after all descendant authority was removed\n");
}

unsafe fn prepare<'a>(
    frames: &'a mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    output: fn(&[u8]) -> bool,
) -> (alloc::boxed::Box<Session>, Frames<'a>) {
    let program = Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    };
    let supervision: [_; LAUNCHES] = core::array::from_fn(|_| Supervision {
        owner: 0,
        peer: 0,
        disk: false,
        device: None,
        keyboard: false,
        framebuffer: None,
        program: Program {
            executable: program.executable,
            arguments: [0, 0],
        },
        frame_limit: usize::MAX,
    });
    let config = Config {
        executables: &[],
        dynamic_tasks: 0,
        frame_limit: usize::MAX,
        endpoints: &[],
        links: &[],
        supervision: &supervision,
        channel_limits: &[],
        cpu_limits: &[],
        clock_readers: &[],
        memory: &[],
    };
    let (mut session, _) = startup::prepare(2, &config, frames.allocated(), output).unwrap();
    let mut frames = Frames::new(frames, session.memory.funding.capacity(), usize::MAX).unwrap();
    for slot in 0..2 {
        // SAFETY: Fresh prepared root, exclusive frame bank; never entered.
        unsafe { admission::admit(&mut session, layout, image, &program, &mut frames, slot) }
            .unwrap();
    }
    // Two branches fill today's 31 task slots without exceeding the wire's
    // per-spawn loan capacity. Both their children must join the same sweep.
    for branch in (0..LAUNCHES).step_by(GROUP) {
        let count = (LAUNCHES - branch - 1).min(abi::MAX_SPAWN_LOANS);
        for offset in 0..count {
            session.launches[branch].delegated[offset] = session.launches[branch + 1 + offset]
                .model
                .launch(0)
                .unwrap();
        }
        session.launches[branch].delegated_count = count;
    }
    for index in 0..LAUNCHES {
        let model = &mut session.launches[index].model;
        let owner = model.owner;
        model
            .request(owner, model.launch(owner).unwrap(), 0)
            .unwrap();
        let mut steps = 0;
        while session.launches[index].model.pending().is_some() {
            session.deferred.remove(Work::Admission, index);
            // SAFETY: Production admission commits the same validated tree loans.
            unsafe { spawn(&mut session, &mut frames, layout, image, &program, index) };
            steps += 1;
            assert!(steps < 512);
        }
        assert!(session.tasks[index + 2].space.is_some());
    }
    for (index, launch) in session.launches.iter().enumerate() {
        assert_eq!(
            launch.model.owner,
            if index.is_multiple_of(GROUP) {
                0
            } else {
                (index / GROUP) * GROUP + 2
            }
        );
    }
    (session, frames)
}

fn receive_on(session: &mut Session, slot: usize, handle: u64) {
    let mut event = Event::Continue;
    assert_eq!(
        syscalls::ipc::dispatch(
            session,
            slot,
            [
                abi::IPC_RECEIVE,
                handle,
                USER_STACK,
                abi::MAX_MESSAGE as u64
            ],
            arch::ticks(),
            &mut event
        ),
        Ok(0)
    );
    assert!(matches!(event, Event::Block));
}
