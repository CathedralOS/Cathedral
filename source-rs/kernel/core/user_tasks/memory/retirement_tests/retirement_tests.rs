// Real-frame retirement: explicit release/death, then queued IPC discard with slot reuse.
use super::*;
use crate::user_tasks::execution::deferred::Work;
use crate::user_tasks::{Config, Executable, Exit, Program, admission, lifecycle, startup};
use crate::{
    ipc::EndpointSpec,
    memory::FrameAllocator,
    scheduler::{Event, TaskState},
};
use cathedral_arch::{BootLayout, ImageRange};

pub(super) unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &BootLayout,
    image: ImageRange,
    output: fn(&[u8]) -> bool,
) {
    // SAFETY: Each inactive fixture returns every frame before the next begins.
    unsafe {
        for death in [false, true] {
            release::exercise(frames, layout, image, death);
        }
        escrow::exercise(frames, layout, image);
    }
    output(b"object retirement: bounded release, death and escrow custody passed\n");
}

struct Fixture<'a> {
    session: alloc::boxed::Box<Session>,
    frames: Frames<'a>,
    baseline: usize,
    admitted: usize,
}
unsafe fn setup<'a>(
    frames: &'a mut FrameAllocator,
    layout: &BootLayout,
    image: ImageRange,
) -> Fixture<'a> {
    let baseline = frames.allocated();
    let grants = [0, 1, 2].map(|task| wire::Grant {
        task,
        private_pages: wire::MAX_PRIVATE_PAGES,
        shared_pages: wire::MAX_READER_PAGES,
    });
    let endpoints = [1, 1, 2].map(|receiver| EndpointSpec {
        sender: 0,
        receiver,
        revoker: Some(0),
    });
    let config = Config {
        executables: &[],
        dynamic_tasks: 0,
        frame_limit: usize::MAX,
        endpoints: &endpoints,
        links: &[],
        supervision: &[],
        channel_limits: &[],
        cpu_limits: &[],
        clock_readers: &[],
        memory: &grants,
    };
    let (mut session, _) = startup::prepare(3, &config, baseline, |_| true).unwrap();
    let mut frames = Frames::new(frames, session.memory.funding.capacity(), usize::MAX).unwrap();
    let program = Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    };
    // SAFETY: None of these roots executes; the probe bytes are only mapped.
    unsafe {
        for slot in 0..3 {
            admission::admit(&mut session, layout, image, &program, &mut frames, slot).unwrap();
        }
    }
    let admitted = frames.allocated();
    Fixture {
        session,
        frames,
        baseline,
        admitted,
    }
}
fn kill(f: &mut Fixture<'_>, slot: usize) {
    f.session.scheduler.cancel(slot);
    f.session.tasks[slot].report.exit = Some(Exit::Cancelled);
    lifecycle::close(&mut f.session, slot);
}
fn conserved(f: &Fixture<'_>) {
    let private: usize = f
        .session
        .tasks
        .iter()
        .filter_map(|t| t.space.as_ref())
        .map(|s| s.frame_count())
        .sum();
    assert_eq!(
        f.frames.allocated(),
        f.baseline + private + f.session.memory.frames()
    );
    assert_eq!(
        f.session.memory.funding.charged(),
        private + f.session.memory.frames() + f.session.memory.reserved_unbacked()
    );
}
unsafe fn finish(mut f: Fixture<'_>) {
    assert!(!f.session.memory.retirement_pending());
    assert!(!f.session.ipc.memory_retirement_pending());
    assert!(f.session.memory.releases.iter().all(Option::is_none));
    for slot in 0..3 {
        if f.session.tasks[slot].space.is_some() {
            // SAFETY: All fixture roots remain inactive and have no remaining objects.
            unsafe { admission::retire(&mut f.session, slot, &mut f.frames) };
        }
    }
    assert_eq!(f.frames.allocated(), f.baseline);
    assert_eq!(f.session.memory.funding.charged(), 0);
}
