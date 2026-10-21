//! Receiver death must not recycle slots held by current or next discard snapshots.
use super::*;
pub(super) unsafe fn exercise(frames: &mut FrameAllocator, layout: &BootLayout, image: ImageRange) {
    // SAFETY: Fresh inactive fixture owns all roots and physical backing.
    let mut f = unsafe { setup(frames, layout, image) };
    let mut reads = [0; 3];
    for (i, read) in reads.iter_mut().enumerate() {
        // SAFETY: Private owner mapping; freeze before giving out any reference.
        let handle =
            unsafe { allocation::allocate(&mut f.session, &mut f.frames, 0, wire::MAX_PAGES) }
                .unwrap();
        // SAFETY: The inactive owner holds the sole writable alias.
        unsafe { objects::freeze(&mut f.session, 0, handle) }.unwrap();
        let send = f.session.ipc.handle(0, (2 * i) as u64).unwrap();
        let receiver = f.session.ipc.memory_receiver(0, send, 1).unwrap();
        *read = f
            .session
            .memory
            .model
            .reserve_read(0, handle, receiver)
            .unwrap();
        f.session.ipc.send_memory(0, send, b"x", *read);
        // SAFETY: Escrow retains the immutable backing after the owner's alias disappears.
        unsafe { retirement::release(&mut f.session, &mut f.frames, 0, handle) }.unwrap();
    }
    for i in 0..2 {
        let revoke = f.session.ipc.handle(0, 2 * i + 1).unwrap();
        f.session.ipc.revoke(0, revoke).unwrap();
    }
    let before = f.frames.allocated();
    // SAFETY: Consume the first snapshot, partly reclaiming its first object.
    unsafe { process(&mut f.session, &mut f.frames) };
    assert_eq!(before - f.frames.allocated(), retirement::FRAME_BATCH);
    assert!(f.session.memory.escrow_object.is_some());
    assert!(f.session.memory.model.reads[reads[1]].is_some());
    // A later discard stays in IPC while the old snapshot is in flight.
    let revoke = f.session.ipc.handle(0, 5).unwrap();
    f.session.ipc.revoke(0, revoke).unwrap();
    assert!(f.session.ipc.memory_retiring(reads[2]));
    kill(&mut f, 1);
    kill(&mut f, 2);
    let mut steps = 0;
    while f.session.deferred.reclaiming() {
        // Deliberately leave escrow processing parked while receiver teardown runs.
        // SAFETY: Production cleanup must preserve both queued escrow slot identities.
        unsafe { lifecycle::reap(&mut f.session, &mut f.frames) };
        steps += 1;
        assert!(steps < 1024);
        assert!(f.session.memory.model.reads[reads[1]].is_some());
        assert!(f.session.memory.model.reads[reads[2]].is_some());
        conserved(&f);
    }
    // The already-consumed first slot may be reused; neither pending slot may.
    // SAFETY: The surviving owner's root remains prepared and inactive.
    let survivor = unsafe { allocation::allocate(&mut f.session, &mut f.frames, 0, 1) }.unwrap();
    // SAFETY: The inactive owner holds the sole writable alias.
    unsafe { objects::freeze(&mut f.session, 0, survivor) }.unwrap();
    let reused = f.session.memory.model.reserve_read(0, survivor, 0).unwrap();
    assert_eq!(reused, reads[0]);
    let name = f.session.memory.model.publish_read(0, reused);
    steps = 0;
    while f.session.memory.retirement_pending() || f.session.ipc.memory_retirement_pending() {
        let before = f.frames.allocated();
        // SAFETY: Only surrendered snapshot references may be removed.
        unsafe { process(&mut f.session, &mut f.frames) };
        assert!(before - f.frames.allocated() <= retirement::FRAME_BATCH);
        assert!(f.session.memory.model.access(0, name).is_ok());
        conserved(&f);
        steps += 1;
        assert!(steps <= 3 * wire::MAX_PAGES.div_ceil(retirement::FRAME_BATCH));
    }
    assert!(steps > wire::MAX_PAGES.div_ceil(retirement::FRAME_BATCH));
    assert_eq!(f.session.memory.frames(), 1);
    assert!(f.session.memory.model.reads[reads[1]].is_none());
    assert!(f.session.memory.model.reads[reads[2]].is_none());
    // SAFETY: The retained owner and the reused published reader are independent rights.
    unsafe {
        retirement::release(&mut f.session, &mut f.frames, 0, survivor).unwrap();
        retirement::release(&mut f.session, &mut f.frames, 0, name).unwrap();
        finish(f);
    }
}
