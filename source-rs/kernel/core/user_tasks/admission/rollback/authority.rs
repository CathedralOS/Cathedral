//! Peer changes, owner death and immutable object-source custody across suspension.
use super::*;

#[inline(never)]
pub(super) unsafe fn interrupted(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    death: usize,
) {
    // SAFETY: New isolated fixture with explicit parent and separate peer.
    let mut f = unsafe { setup(frames, layout, image, program) };
    for _ in 0..3 {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Start construction through the production lifecycle, retaining authority.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0);
        }
        unpublished(&f);
    }
    let receive = f.session.launches[0].initial[0];
    if death < 2 {
        f.session.scheduler.cancel(death);
        f.session.tasks[death].report.exit = Some(crate::user_tasks::Exit::Cancelled);
        lifecycle::close(&mut f.session, death);
        // SAFETY: Exited fixture task cannot resume; exercise owner-return cleanup too.
        unsafe {
            reclaim(&mut f);
        }
    } else {
        // A receive right with queued data is no longer movable. The independent
        // producer can make it busy while its owner waits for child admission.
        f.session.ipc.send(1, f.producer, b"arrived").unwrap();
    }
    if death != 0 {
        while f.session.launches[0].model.pending().is_some() {
            f.session.deferred.remove(Work::Admission, 0);
            // SAFETY: A dead peer/busy initial right refuses, then retires the draft.
            unsafe {
                lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0);
            }
        }
        assert_eq!(f.session.tasks[0].report.rejected, 1);
    }
    if death == 2 {
        let message = f.session.ipc.receive(0, receive, 64).unwrap().unwrap();
        assert_eq!(&message.bytes[..message.len], b"arrived");
    }
    assert!(f.session.launches[0].draft.is_none());
    assert!(f.session.tasks[2].space.is_none());
    assert_eq!(f.session.scheduler.states()[2], TaskState::Vacant);
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
    assert_eq!(f.session.tasks[0].report.spawned, 0);
    // SAFETY: Remaining initial task(s) are inactive and retained here.
    unsafe {
        finish(f);
    }
}

// An object-backed executable carries a separate kernel reference. If its last
// owner dies mid-build, both the draft and that source reference must disappear.
#[inline(never)]
pub(super) unsafe fn abandoned_object(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    refusing: bool,
) {
    // SAFETY: New inactive fixture and exclusive bank custody.
    let mut f = unsafe { setup(frames, layout, image, program) };
    f.session.tasks[0].memory_call = Some([abi::MEMORY_ALLOCATE, 2, 0]);
    f.session.deferred.insert(Work::Memory, 0);
    // SAFETY: Prepared inactive root; process owns frame/mapping edits.
    unsafe {
        crate::user_tasks::memory::process(&mut f.session, &mut f.source);
    }
    assert!(f.session.tasks[0].memory_call.is_none());
    let region = f
        .session
        .memory
        .model
        .entries
        .iter()
        .position(|entry| entry.owner == Some(0))
        .unwrap();
    let handle = f.session.memory.model.handle(0, region);
    let address = wire::address(f.session.memory.model.mapping(0, region));
    let Executable::Elf(bytes) = program.executable else {
        unreachable!()
    };
    for (page, chunk) in bytes.chunks(wire::PAGE_BYTES).enumerate() {
        // SAFETY: Each copy fits one complete writable page of this allocation.
        assert!(unsafe {
            f.session.tasks[0]
                .space
                .as_ref()
                .unwrap()
                .copy_to_user(address + (page * wire::PAGE_BYTES) as u64, chunk)
        });
    }
    f.session.tasks[0].memory_call = Some([abi::MEMORY_FREEZE, handle, 0]);
    f.session.deferred.insert(Work::Memory, 0);
    // SAFETY: No references into backing survive sealing.
    unsafe {
        crate::user_tasks::memory::process(&mut f.session, &mut f.source);
    }
    f.session.memory.hold(region);
    f.session.launches[0].code = Some(pool::Code::Object { region, role: 0 });
    let pages = [
        f.session.memory.frames_of(region)[0],
        f.session.memory.frames_of(region)[1],
    ];
    let program = Program {
        executable: Executable::Object(&pages),
        arguments: [0, 7],
    };
    let mut steps = 0;
    while f.session.launches[0]
        .draft
        .as_ref()
        .map_or(0, Draft::frames)
        <= 16
    {
        steps += 1;
        assert!(steps < image.bytes.div_ceil(4096) as usize + 128);
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Kernel reference retains immutable source pages across these steps.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, &program, 0);
        }
    }
    assert!(f.session.launches[0].draft.is_some());
    if refusing {
        f.session.ipc.send(1, f.producer, b"busy").unwrap();
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Refusal retains the same immutable object reference as construction.
        unsafe { lifecycle::spawn(&mut f.session, &mut f.source, layout, image, &program, 0) };
        assert_eq!(f.session.launches[0].refusal, Some(abi::BUSY));
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Begin partial rollback, then withdraw its owner below.
        unsafe { lifecycle::spawn(&mut f.session, &mut f.source, layout, image, &program, 0) };
        assert!(f.session.launches[0].draft.is_some());
    }
    f.session.scheduler.cancel(0);
    f.session.tasks[0].report.exit = Some(crate::user_tasks::Exit::Cancelled);
    lifecycle::close(&mut f.session, 0);
    // SAFETY: Owner cannot resume; abort draft before settling its source reference.
    unsafe {
        reclaim(&mut f);
        crate::user_tasks::memory::process(&mut f.session, &mut f.source);
    }
    assert!(f.session.launches[0].draft.is_none());
    assert!(f.session.launches[0].code.is_none());
    assert_eq!(f.session.memory.frames(), 0);
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
    // SAFETY: Only the inactive initial peer remains.
    unsafe {
        finish(f);
    }
}
