//! Real-frame executable-reference bursts: retained objects cross bitmap words,
//! share repeated kernel references, and leave one live owner alias untouched.
use super::*;

pub(super) unsafe fn exercise(
    session: &mut Session,
    frames: &mut Frames<'_>,
    output: fn(&[u8]) -> bool,
) {
    const OBJECTS: usize = 70;
    let baseline = frames.allocated();
    let charged = session.memory.funding.charged();
    let mut regions = [0; OBJECTS];
    let mut survivor = 0;
    frames.remaining = usize::MAX;
    let large_objects = ((wire::MAX_PRIVATE_PAGES - OBJECTS) / wire::MAX_PAGES).min(5);
    assert!(large_objects >= 2);
    for (index, region) in regions.iter_mut().enumerate() {
        // Cross the bitmap word boundary, then retain several maximal objects
        // without increasing the account's unchanged aggregate funding.
        let pages = if index < OBJECTS - large_objects {
            1
        } else {
            wire::MAX_PAGES
        };
        // SAFETY: Fixture owns this prepared inactive root and every physical frame.
        let handle = unsafe { allocation::allocate(session, frames, 0, pages) }.unwrap();
        *region = session.memory.model.access(0, handle).unwrap();
        // SAFETY: No user context or mutable copy exists during sealing.
        unsafe { objects::freeze(session, 0, handle) }.unwrap();
        session.memory.hold(*region);
        session.memory.hold(*region);
        if index + 1 == OBJECTS {
            survivor = handle;
        } else {
            // SAFETY: Kernel references retain backing after the only alias is removed.
            unsafe { retirement::release(session, frames, 0, handle) }.unwrap();
        }
    }
    assert!(regions[0] < 64 && regions[OBJECTS - 1] >= 64);
    for region in regions {
        session.memory.unref_later(region);
        session.memory.unref_later(region);
    }
    output(b"reference retirement: retained-object burst\n");
    let mut steps = 0;
    while session.memory.retirement_pending() {
        let before = frames.allocated();
        // SAFETY: Production settlement, exclusively owned inactive machine state.
        unsafe { process(session, frames) };
        steps += 1;
        assert!(before - frames.allocated() <= retirement::FRAME_BATCH);
        assert_eq!(frames.allocated() - baseline, session.memory.frames());
        assert_eq!(
            session.memory.funding.charged() - charged,
            session.memory.frames()
        );
    }
    assert!(steps >= OBJECTS);
    session.timing.report(output);
    assert_eq!(frames.allocated(), baseline + wire::MAX_PAGES);
    let region = session.memory.model.access(0, survivor).unwrap();
    assert_eq!(session.memory.model.entries[region].read_refs, 0);
    let address = wire::address(session.memory.model.mapping(0, region));
    let space = session.tasks[0].space.as_ref().unwrap();
    let mut byte = [1];
    // SAFETY: Surviving sealed owner alias must remain readable after reference retirement.
    assert!(unsafe { space.copy_from_user(address, &mut byte) });
    assert_eq!(byte, [0]);
    assert!(!space.writable_range(address, 1));
    // SAFETY: Final alias/ref goes away before backing is returned.
    unsafe { retirement::release(session, frames, 0, survivor) }.unwrap();
    assert_eq!(frames.allocated(), baseline);
    assert_eq!(session.memory.funding.charged(), charged);
    assert_eq!(session.memory.frames(), 0);
    output(b"reference retirement: burst refunded, live owner preserved\n");
}
