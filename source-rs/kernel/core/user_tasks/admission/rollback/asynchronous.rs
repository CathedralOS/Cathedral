//! Exercise real syscall copy-in/out and deferred construction while the parent
//! is blocked on unrelated work. Completion must not touch that wait or context.
use super::*;
use crate::scheduler::Event;
use crate::user_tasks::syscalls::tasks::dispatch;

#[inline(never)]
pub(super) unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    outcome: usize,
) {
    // SAFETY: Exclusive fixture custody; none of these roots ever execute.
    let mut f = unsafe { setup(frames, layout, image, program) };
    let receive = f.session.launches[0].initial[0];
    f.session.launches[0].model.failed();
    let grant = f.session.launches[0].model.launch(0).unwrap();
    let mut record = [0; abi::SPAWN_TREE_BYTES];
    record[..8].copy_from_slice(&7u64.to_le_bytes());
    record[8..16].copy_from_slice(&1u64.to_le_bytes());
    record[16..24].copy_from_slice(&receive.to_le_bytes());
    // SAFETY: Checked copy into retained inactive parent backing.
    assert!(unsafe {
        f.session.tasks[0]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(USER_DATA, &record)
    });
    let mut event = Event::Continue;
    let receipt = dispatch(
        &mut f.session,
        0,
        (abi::TASK_SPAWN_BEGIN, grant, USER_DATA, 0),
        &mut event,
        0,
    )
    .unwrap();
    assert!(receipt > 0 && receipt <= i64::MAX as u64);
    assert!(matches!(event, Event::Continue));
    assert_eq!(poll(&mut f, 1, receipt, USER_DATA, 0), Err(abi::BAD_HANDLE));
    assert_eq!(
        poll(&mut f, 0, receipt, USER_DATA, 0),
        Err(abi::WOULD_BLOCK)
    );
    assert_eq!(
        poll(&mut f, 0, receipt, USER_CODE, 1),
        Err(abi::BAD_ADDRESS)
    );
    assert!(f.session.launches[0].model.admission_ready().is_ok());

    f.session.tasks[0].context.set_result(0x1234);
    let saved = f.session.tasks[0].context.syscall();
    assert_eq!(f.session.scheduler.advance(Event::Yield, 0, false), Some(0));
    f.session.scheduler.park(Event::Block, 0);
    for _ in 0..3 {
        // SAFETY: Same private construction, retaining all authority and frames.
        unsafe { advance_one(&mut f, layout, image, program) };
        unpublished(&f);
    }
    assert!(f.source.allocated() > f.admitted);
    match outcome {
        1 => assert_eq!(
            poll(&mut f, 0, receipt, USER_DATA, 1),
            Err(abi::WOULD_BLOCK)
        ),
        2 => f.session.ipc.send(1, f.producer, b"busy").unwrap(),
        3 | 4 => {
            let dead = outcome - 3;
            f.session.scheduler.cancel(dead);
            f.session.tasks[dead].report.exit = Some(crate::user_tasks::Exit::Cancelled);
            lifecycle::close(&mut f.session, dead);
            // SAFETY: Withdrawn inactive roots retain custody through all batches.
            unsafe { reclaim(&mut f) };
        }
        _ => {}
    }
    let mut steps = 0;
    while f.session.launches[0].model.pending().is_some() {
        steps += 1;
        assert!(steps < 256);
        // SAFETY: Sole boot CPU, unpublished draft or bounded rollback.
        unsafe { advance_one(&mut f, layout, image, program) };
        assert_eq!(f.session.scheduler.states()[0], TaskState::Blocked);
        assert_eq!(f.session.tasks[0].context.syscall(), saved);
    }
    if outcome == 3 {
        assert!(!f.session.launches[0].model.has_receipt(0, receipt));
        assert_eq!(f.session.scheduler.states()[0], TaskState::Vacant);
    } else {
        // Bad copyout cannot consume a completed result. A late cancel returns
        // the committed child; failures become visible only after full rollback.
        assert_eq!(
            poll(&mut f, 0, receipt, USER_CODE, 1),
            Err(abi::BAD_ADDRESS)
        );
        assert_eq!(poll(&mut f, 0, receipt, USER_DATA, 1), Ok(0));
        assert_eq!(poll(&mut f, 0, receipt, USER_DATA, 0), Err(abi::BAD_HANDLE));
        let mut result = [0; 16];
        // SAFETY: Parent backing remains live; checked read of the copied result.
        assert!(unsafe {
            f.session.tasks[0]
                .space
                .as_ref()
                .unwrap()
                .copy_from_user(USER_DATA, &mut result)
        });
        let status = u64::from_le_bytes(result[..8].try_into().unwrap());
        let child = u64::from_le_bytes(result[8..].try_into().unwrap());
        if outcome == 0 {
            assert_eq!(status, 0);
            assert!(child > 0 && child <= i64::MAX as u64);
            assert_eq!(f.session.scheduler.states()[2], TaskState::Ready);
            assert_eq!(
                f.session.ipc.check(0, receive, Right::Receive),
                Err(abi::BAD_HANDLE)
            );
        } else {
            assert_eq!(
                status,
                if outcome == 2 {
                    abi::BUSY
                } else if outcome == 4 {
                    abi::PEER_CLOSED
                } else {
                    abi::REVOKED
                }
            );
            assert_eq!(child, 0);
            assert!(f.session.tasks[2].space.is_none());
            assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
            if outcome != 4 {
                restored(&f);
            }
        }
    }
    // SAFETY: No fixture context ran; every surviving root is retained here.
    unsafe { finish(f) };
}

fn poll(
    f: &mut fixture::Fixture<'_>,
    caller: usize,
    receipt: u64,
    address: u64,
    cancel: u64,
) -> Result<u64, u64> {
    let mut event = Event::Continue;
    let result = dispatch(
        &mut f.session,
        caller,
        (abi::TASK_SPAWN_RESULT, receipt, address, cancel),
        &mut event,
        0,
    );
    assert!(matches!(event, Event::Continue));
    result
}

unsafe fn advance_one(
    f: &mut fixture::Fixture<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
) {
    f.session.deferred.remove(Work::Admission, 0);
    // SAFETY: Caller retains inactive machine/frame custody for this fixture.
    unsafe { lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0) };
}
