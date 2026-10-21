//! Last release blocks until refund; death inherits that partial cleanup without resuming it.
use super::*;
pub(super) unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &BootLayout,
    image: ImageRange,
    death: bool,
) {
    // SAFETY: Fresh inactive roots and exclusive frame custody.
    let mut f = unsafe { setup(frames, layout, image) };
    // SAFETY: No user borrows; fixture owns this prepared root.
    let handle =
        unsafe { allocation::allocate(&mut f.session, &mut f.frames, 0, wire::MAX_PAGES) }.unwrap();
    let index = f.session.memory.model.access(0, handle).unwrap();
    let address = wire::address(f.session.memory.model.mapping(0, index));
    assert_eq!(f.session.scheduler.advance(Event::Yield, 0, false), Some(0));
    f.session.scheduler.park(Event::Block, 0);
    f.session.tasks[0].memory_call = Some([abi::MEMORY_RELEASE, handle, 0]);
    f.session.deferred.insert(Work::Memory, 0);
    // SAFETY: Production deferred call removes aliases before its first frame batch.
    unsafe { process(&mut f.session, &mut f.frames) };
    assert_eq!(
        f.frames.allocated(),
        f.admitted + wire::MAX_PAGES - retirement::FRAME_BATCH
    );
    assert!(f.session.memory.model.entries[index].retiring);
    assert_eq!(f.session.memory.model.charged(0)[0], wire::MAX_PAGES);
    assert_eq!(
        f.session.memory.model.access(0, handle),
        Err(abi::BAD_HANDLE)
    );
    let mut byte = [0];
    // SAFETY: The removed mapping must already reject checked access.
    assert!(!unsafe {
        f.session.tasks[0]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut byte)
    });
    conserved(&f);
    // A different task can allocate while this object is partly retired, but
    // cannot reuse its metadata slot or consume its still-charged backing.
    // SAFETY: Independent inactive peer root and funded account.
    let peer = unsafe { allocation::allocate(&mut f.session, &mut f.frames, 1, 1) }.unwrap();
    assert_ne!(f.session.memory.model.access(1, peer).unwrap(), index);
    // SAFETY: Only the peer's one-page object is surrendered here.
    unsafe { retirement::release(&mut f.session, &mut f.frames, 1, peer) }.unwrap();
    if death {
        kill(&mut f, 0);
    }
    let mut steps = 0;
    while if death {
        f.session.deferred.reclaiming()
    } else {
        f.session.tasks[0].memory_call.is_some()
    } {
        let before = f.frames.allocated();
        // SAFETY: Both production routes retain the inactive retired root/object.
        unsafe {
            if death {
                lifecycle::reap(&mut f.session, &mut f.frames);
            } else {
                process(&mut f.session, &mut f.frames);
            }
        }
        steps += 1;
        assert!(steps < 512);
        assert!(before - f.frames.allocated() <= retirement::FRAME_BATCH);
        conserved(&f);
        assert_eq!(f.session.scheduler.states()[1], TaskState::Ready);
        if !death && f.session.tasks[0].memory_call.is_some() {
            assert_eq!(f.session.scheduler.states()[0], TaskState::Blocked);
            assert!(f.session.memory.model.entries[index].retiring);
        }
    }
    assert!(!f.session.memory.model.entries[index].retiring);
    assert_eq!(f.session.memory.model.charged(0)[0], 0);
    if !death {
        assert_eq!(steps + 1, wire::MAX_PAGES.div_ceil(retirement::FRAME_BATCH));
        assert_eq!(f.session.scheduler.states()[0], TaskState::Ready);
        assert_eq!(f.frames.allocated(), f.admitted);
    } else {
        assert_eq!(f.session.scheduler.states()[0], TaskState::Vacant);
    }
    // SAFETY: Remaining inactive tasks have no page-object aliases or references.
    unsafe { finish(f) };
}
