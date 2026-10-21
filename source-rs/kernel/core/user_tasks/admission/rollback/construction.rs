//! Construct, refuse and withdraw at every physical allocation and phase boundary.
use super::*;

#[inline(never)]
pub(super) unsafe fn physical(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    framebuffer: Option<cathedral_kernel_contracts::framebuffer::Framebuffer>,
    output: fn(&[u8]) -> bool,
) {
    // SAFETY: Exclusive machine/frame custody for this entire fixture.
    let mut f = unsafe { setup(frames, layout, image, program) };
    // Only the private draft may borrow the real boot-approved aperture. No
    // fixture root executes or accesses device backing, including across aborts.
    f.session.launches[0].framebuffer = framebuffer;
    let mut steps = 0;
    let mut kernel_steps = 0;
    let mut framebuffer_steps = 0;
    loop {
        steps += 1;
        if f.session.launches[0]
            .draft
            .as_ref()
            .is_some_and(Draft::preparing_kernel)
        {
            kernel_steps += 1;
        }
        if f.session.launches[0]
            .draft
            .as_ref()
            .is_some_and(Draft::preparing_framebuffer)
        {
            framebuffer_steps += 1;
        }
        // SAFETY: Immutable ELF retained; child remains outside runnable context storage.
        let ready =
            unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) }.unwrap();
        unpublished(&f);
        if ready {
            break;
        }
        // The smoke kernel embeds the lab executables; its supervisor image can
        // need more batches as fixtures grow. Keep only a generous stall guard.
        assert!(steps < image.bytes.div_ceil(4096) as usize + 128);
    }
    assert!(steps > 8); // 64 image pages + stack, not just a tiny probe.
    #[cfg(target_arch = "x86_64")]
    assert!(kernel_steps > 1); // The heap alone requires several supervisor batches.
    #[cfg(target_arch = "aarch64")]
    assert_eq!(kernel_steps, 1); // This backend borrows a permanent branch at creation.
    if let Some(fb) = framebuffer {
        assert_eq!(
            framebuffer_steps,
            (fb.bytes as usize / 4096)
                .div_ceil(crate::user_tasks::admission::construction::FRAMEBUFFER_PAGE_BATCH)
        );
    }
    output(b"staged admission: complete construction observations\n");
    f.session.timing.report(output);
    let required = f.source.allocated() - f.admitted;
    // SAFETY: Even a complete draft is private until final authority commit.
    unsafe {
        discard_draft(&mut f);
    }
    restored(&f);
    for limit in 0..required {
        f.session.launches[0].frame_limit = limit;
        loop {
            // SAFETY: Same inactive roots; deterministic failure at every frame boundary.
            match unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) } {
                Ok(false) => unpublished(&f),
                Ok(true) => panic!("admission exceeded total frame limit"),
                Err(Error::Memory(arch::MemoryError::OutOfFrames)) => break,
                Err(other) => panic!("unexpected refusal: {other:?}"),
            }
        }
        // SAFETY: A failed advance retains its charged draft until rollback completes.
        unsafe { discard_draft(&mut f) };
        restored(&f);
    }
    output(b"staged admission: frame refusals passed\n");
    f.session.launches[0].frame_limit = usize::MAX;
    for boundary in 1..=steps {
        for _ in 0..boundary {
            // SAFETY: Retained source, same private child root.
            unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) }.unwrap();
            unpublished(&f);
        }
        // SAFETY: Exercise withdrawal at every coherent stage, including fully built.
        unsafe {
            discard_draft(&mut f);
        }
        restored(&f);
    }
    output(b"staged admission: withdrawals passed\n");
    let old = f.session.launches[0].initial[0];
    while f.session.launches[0].model.pending().is_some() {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Exercise the actual lifecycle commit with inactive parent/peer roots.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0);
        }
    }
    assert_eq!(f.session.tasks[0].report.spawned, 1);
    assert_eq!(f.session.scheduler.states()[2], TaskState::Ready);
    assert_eq!(
        f.session.ipc.check(0, old, Right::Receive),
        Err(abi::BAD_HANDLE)
    );
    let space = f.session.tasks[2].space.as_ref().unwrap();
    // Device mappings never enter the checked-copy RAM ledger.
    assert!(!space.writable_range(cathedral_kernel_contracts::framebuffer::USER_ADDRESS, 1));
    let mut byte = [0];
    // SAFETY: Live inactive committed root; copies validate the complete ranges.
    unsafe {
        assert!(space.copy_from_user(USER_CODE, &mut byte));
        assert_eq!(byte, [0xc3]);
        for page in 0..63 {
            let mut tail = [1; 32];
            assert!(space.copy_from_user(USER_DATA + page * 4096 + 4064, &mut tail));
            assert_eq!(tail, [0; 32]);
        }
    }
    assert!(!space.writable_range(USER_CODE, 1));
    assert!(space.writable_range(USER_DATA, 1));
    f.session.scheduler.cancel(0);
    f.session.tasks[0].report.exit = Some(crate::user_tasks::Exit::Cancelled);
    lifecycle::close(&mut f.session, 0);
    let mut batches = 0;
    while f.session.deferred.reclaiming() {
        let before = f.source.allocated();
        // SAFETY: Production cancellation withdraws the child before the first yield.
        unsafe {
            lifecycle::reap(&mut f.session, &mut f.source);
        }
        batches += 1;
        assert!(batches < 256);
        assert!(before - f.source.allocated() <= 8);
        assert_eq!(f.session.scheduler.states()[1], TaskState::Ready);
        assert_eq!(
            f.session.ipc.send(1, f.producer, b"late"),
            Err(abi::PEER_CLOSED)
        );
        if f.session.deferred.reclaiming() {
            // Neither the parent nor child slot/outcome is reusable mid-wave.
            assert_eq!(f.session.scheduler.states()[0], TaskState::Exited);
            assert_eq!(f.session.scheduler.states()[2], TaskState::Exited);
            assert_eq!(f.session.tasks[0].report.reaped, 0);
        }
    }
    assert!(batches > 8);
    assert_eq!(f.session.scheduler.states()[0], TaskState::Vacant);
    assert_eq!(f.session.scheduler.states()[2], TaskState::Vacant);
    assert_eq!(f.session.tasks[0].report.reaped, 1);
    // SAFETY: No fixture context was ever activated.
    unsafe {
        finish(f);
    }
}
