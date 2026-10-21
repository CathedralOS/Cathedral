//! Late launch refusal stays private and blocked through bounded rollback.
use super::*;

#[inline(never)]
pub(super) unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    failure: usize,
) {
    use crate::scheduler::Event;
    // SAFETY: Fresh inactive fixture; no draft or initial task is ever executed.
    let mut f = unsafe { setup(frames, layout, image, program) };
    // Build the entire image, including all reserved runtime tables, privately.
    // SAFETY: Immutable source and exclusive frame/root custody until retirement.
    while !unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) }.unwrap() {}
    let required = f.source.allocated() - f.admitted;
    assert!(required > 64);
    let receive = f.session.launches[0].initial[0];
    let expected = if failure == 0 {
        // Rebuild through lifecycle with a refusal at the final allocation.
        // SAFETY: Complete draft remains unpublished and exclusively owned.
        unsafe { discard_draft(&mut f) };
        f.session.launches[0].frame_limit = required - 1;
        abi::NO_MEMORY
    } else {
        f.session.ipc.send(1, f.producer, b"busy").unwrap();
        abi::BUSY
    };
    assert_eq!(f.session.scheduler.advance(Event::Yield, 0, false), Some(0));
    f.session.scheduler.park(Event::Block, 0);
    let mut steps = 0;
    while f.session.launches[0].refusal.is_none() {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Production admission detects error without publishing the child.
        unsafe { lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0) };
        steps += 1;
        assert!(steps < image.bytes.div_ceil(4096) as usize + 128);
        assert!(f.session.launches[0].model.pending().is_some());
        unpublished(&f);
    }
    assert_eq!(f.session.launches[0].refusal, Some(expected));
    let retained = f.source.allocated() - f.admitted;
    assert!(retained > 8);
    let mut batches = 0;
    while f.session.launches[0].model.pending().is_some() {
        assert_eq!(f.session.tasks[0].report.rejected, 0);
        assert_eq!(f.session.scheduler.states()[0], TaskState::Blocked);
        let grant = f.session.launches[0].model.launch(0).unwrap();
        assert_eq!(
            f.session.launches[0].model.request(0, grant, 9),
            Err(abi::BUSY)
        );
        // A peer remains selectable between rollback visits while the owner waits.
        assert_eq!(f.session.scheduler.advance(Event::Yield, 0, false), Some(1));
        f.session.scheduler.park(Event::Yield, 0);
        let before = f.source.allocated();
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Same inactive draft; production rollback owns every retained frame.
        unsafe { lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0) };
        batches += 1;
        assert!(batches <= retained.div_ceil(8));
        assert!(before - f.source.allocated() <= 8);
        if failure == 2 && batches == 2 {
            f.session.scheduler.cancel(0);
            f.session.tasks[0].report.exit = Some(crate::user_tasks::Exit::Cancelled);
            lifecycle::close(&mut f.session, 0);
            assert!(f.session.launches[0].refusal.is_none());
            assert!(!f.session.deferred.has(Work::Admission));
            // SAFETY: Owner death transfers the partial draft to abandonment.
            unsafe { reclaim(&mut f) };
            assert_eq!(f.session.tasks[0].report.rejected, 0);
            assert_eq!(f.session.scheduler.states()[0], TaskState::Vacant);
            assert_eq!(f.session.scheduler.states()[1], TaskState::Ready);
            break;
        }
        if f.session.launches[0].model.pending().is_some() {
            unpublished(&f);
            assert_eq!(f.session.launches[0].refusal, Some(expected));
        }
    }
    assert!(f.session.launches[0].draft.is_none());
    assert!(f.session.launches[0].refusal.is_none());
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
    assert_eq!(f.session.scheduler.states()[2], TaskState::Vacant);
    if failure != 2 {
        assert_eq!(batches, retained.div_ceil(8));
        restored(&f);
        assert_eq!(f.session.tasks[0].report.rejected, 1);
        assert_eq!(f.session.scheduler.states()[0], TaskState::Ready);
        assert!(f.session.ipc.check(0, receive, Right::Receive).is_ok());
        if failure == 1 {
            let message = f.session.ipc.receive(0, receive, 64).unwrap().unwrap();
            assert_eq!(&message.bytes[..message.len], b"busy");
        }
        // Retrying after a completed refusal must not reuse the old failure state.
        let grant = f.session.launches[0].model.launch(0).unwrap();
        f.session.launches[0].model.request(0, grant, 9).unwrap();
        f.session.launches[0].frame_limit = usize::MAX;
        f.session.launches[0].initial[0] = receive;
        f.session.launches[0].initial_count = 1;
        while f.session.launches[0].model.pending().is_some() {
            f.session.deferred.remove(Work::Admission, 0);
            // SAFETY: Ordinary production retry; private source remains live.
            unsafe { lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0) };
        }
        assert_eq!(f.session.tasks[0].report.spawned, 1);
        assert_eq!(f.session.scheduler.states()[2], TaskState::Ready);
        f.session.scheduler.cancel(0);
        f.session.tasks[0].report.exit = Some(crate::user_tasks::Exit::Cancelled);
        lifecycle::close(&mut f.session, 0);
        // SAFETY: Parent and admitted child are withdrawn before reclaiming.
        unsafe { reclaim(&mut f) };
    }
    // SAFETY: The surviving peer never ran; all draft/child frames were retired.
    unsafe { finish(f) };
}
