//! Private resumable address-space construction. No task context, grant or runnable
//! state is installed here. Lifecycle revalidates authority before publication.
use super::{Error, Frames, Program, Report, Session, image};
use crate::user_tasks::address_space::{LoadBudget, LoadCursor, USER_STACK_TOP, UserSpace};
use crate::user_tasks::execution::timing::{self, Phase as TimedPhase};
use cathedral_arch::{self as arch, Context};

pub(super) const PAGE_BATCH: usize = 8;
// File copies dominate the observed image-loading bursts. Bound their bytes
// separately while retaining eight cheap zero-only pages; neither is WCET.
const FILE_BYTES_BATCH: usize = 2 * arch::memory::PAGE as usize;
// Leaf edits are lighter than image copying; this is a work allowance, not WCET.
const KERNEL_PAGE_BATCH: usize = 64;
pub(super) const FRAMEBUFFER_PAGE_BATCH: usize = 64;
enum Phase {
    Kernel,
    Image,
    Regions,
    Framebuffer,
    Ready,
}
pub(in crate::user_tasks) struct Draft {
    space: UserSpace,
    plan: image::Plan,
    cursor: LoadCursor,
    framebuffer_offset: u64,
    phase: Phase,
}
impl Draft {
    pub(in crate::user_tasks) fn frames(&self) -> usize {
        self.space.frame_count()
    }
    #[cfg(feature = "memory-lab")]
    pub(super) fn preparing_kernel(&self) -> bool {
        matches!(self.phase, Phase::Kernel)
    }
    #[cfg(feature = "memory-lab")]
    pub(super) fn preparing_framebuffer(&self) -> bool {
        matches!(self.phase, Phase::Framebuffer)
    }
}

/// # Safety
/// Boot stack, sole CPU/kernel root/IRQs off. Launch retains immutable executable
/// custody across calls; caller rechecks its live owner/peer before every step.
/// Errors retain any charged private draft; caller must retire it before reuse.
pub(in crate::user_tasks) unsafe fn advance(
    session: &mut Session,
    layout: &arch::BootLayout,
    image_range: arch::ImageRange,
    program: &Program<'_>,
    frames: &mut Frames<'_>,
    index: usize,
) -> Result<bool, Error> {
    let child = session.launches[index].model.child;
    assert!(session.tasks[child].space.is_none());
    let used = session.launches[index]
        .draft
        .as_ref()
        .map_or(0, Draft::frames);
    let previous = frames.remaining;
    frames.remaining = session.launches[index]
        .frame_limit
        .saturating_sub(used)
        .min(session.memory.funding.available(child));
    let baseline = frames.allocated();
    let phase = match session.launches[index]
        .draft
        .as_ref()
        .map(|draft| &draft.phase)
    {
        None => Some(TimedPhase::DecodeRoot),
        Some(Phase::Kernel) => Some(TimedPhase::KernelMappings),
        Some(Phase::Image) => Some(TimedPhase::ImageBatch),
        Some(Phase::Regions) => session
            .memory
            .model
            .enabled(child)
            .then_some(TimedPhase::RegionTables),
        Some(Phase::Framebuffer) => session.launches[index]
            .framebuffer
            .map(|_| TimedPhase::Framebuffer),
        Some(Phase::Ready) => unreachable!("completed draft advanced"),
    };
    let start = timing::start();
    // SAFETY: Same private-root and executable custody as advance.
    let result = unsafe { step(session, layout, image_range, program, frames, index) };
    if let Some(phase) = phase {
        session.timing.record(phase, start);
    }
    let allocated = frames.allocated() - baseline;
    session
        .memory
        .funding
        .charge(child, allocated)
        .expect("construction exceeded funding");
    frames.remaining = previous;
    result
}

unsafe fn step(
    session: &mut Session,
    layout: &arch::BootLayout,
    image_range: arch::ImageRange,
    program: &Program<'_>,
    frames: &mut Frames<'_>,
    index: usize,
) -> Result<bool, Error> {
    let child = session.launches[index].model.child;
    if session.launches[index].draft.is_none() {
        let start = timing::start();
        // SAFETY: The launch holds the source object or immutable boot template.
        let decoded = unsafe { image::decode(program) };
        session.timing.record(TimedPhase::Decode, start);
        let (plan, _) = decoded?;
        let start = timing::start();
        // SAFETY: Private root; begin refunds its own partial tree on failure.
        let created = unsafe { UserSpace::begin(layout, image_range, &plan.segments, frames) };
        session.timing.record(TimedPhase::Root, start);
        let space = created.map_err(Error::Memory)?;
        session.launches[index].draft = Some(Draft {
            space,
            plan,
            cursor: LoadCursor::default(),
            framebuffer_offset: 0,
            phase: Phase::Kernel,
        });
        return Ok(false);
    }
    let framebuffer = session.launches[index].framebuffer;
    let draft = session.launches[index].draft.as_mut().unwrap();
    // SAFETY: The draft is inactive and exclusively owned; only its next phase
    // changes. Every table/backing allocation is captured by the space ledger.
    unsafe {
        match draft.phase {
            Phase::Kernel => {
                if draft
                    .space
                    .prepare_kernel(frames, KERNEL_PAGE_BATCH)
                    .map_err(Error::Memory)?
                {
                    draft.phase = Phase::Image;
                }
            }
            Phase::Image => {
                #[cfg(feature = "memory-lab")]
                let mut copied_bytes = 0;
                let loaded = draft.space.load_pages(
                    &draft.plan.segments,
                    image::contents(program),
                    &mut draft.cursor,
                    LoadBudget {
                        pages: PAGE_BATCH,
                        file_bytes: FILE_BYTES_BATCH,
                    },
                    frames,
                    #[cfg(feature = "memory-lab")]
                    Some(&mut |stage, elapsed| {
                        use crate::user_tasks::address_space::LoadStage;
                        let phase = match stage {
                            LoadStage::Backing => TimedPhase::ImageBacking,
                            LoadStage::Populate { file_bytes } => {
                                copied_bytes += file_bytes;
                                if file_bytes == 0 {
                                    TimedPhase::ImageZero
                                } else {
                                    TimedPhase::ImageFile
                                }
                            }
                            LoadStage::Publish => TimedPhase::ImagePublish,
                        };
                        session.timing.record_elapsed(phase, elapsed);
                    }),
                );
                #[cfg(feature = "memory-lab")]
                assert!(
                    copied_bytes <= FILE_BYTES_BATCH,
                    "image turn exceeded file-byte allowance"
                );
                if loaded.map_err(Error::Memory)? {
                    draft.phase = Phase::Regions;
                }
            }
            Phase::Regions => {
                if session.memory.model.enabled(child) {
                    draft.space.prepare_regions(frames).map_err(Error::Memory)?;
                }
                draft.phase = Phase::Framebuffer;
            }
            Phase::Framebuffer => {
                if let Some(fb) = framebuffer
                    && !draft
                        .space
                        .map_framebuffer_pages(
                            fb,
                            &mut draft.framebuffer_offset,
                            FRAMEBUFFER_PAGE_BATCH,
                            frames,
                        )
                        .map_err(Error::Memory)?
                {
                    return Ok(false);
                }
                draft.phase = Phase::Ready;
            }
            Phase::Ready => unreachable!("completed draft was neither published nor aborted"),
        }
    }
    Ok(matches!(draft.phase, Phase::Ready))
}

/// # Safety
/// Refused or withdrawn launch, private root never activated. Caller keeps the
/// launch unavailable until every frame and the retained source are settled.
pub(in crate::user_tasks) unsafe fn abort_step(
    session: &mut Session,
    frames: &mut Frames<'_>,
    index: usize,
    budget: usize,
) -> bool {
    let start = timing::start();
    let child = session.launches[index].model.child;
    let Some(draft) = session.launches[index].draft.as_mut() else {
        return true;
    };
    let before = draft.frames();
    // SAFETY: No context or page-object aliases were published into this draft.
    let done = unsafe { draft.space.release_pages(frames, budget) };
    session
        .memory
        .funding
        .refund(child, before - draft.frames());
    if done {
        session.launches[index].draft = None;
    }
    session.timing.record(TimedPhase::Rollback, start);
    done
}

/// Final serialized commit only: caller has revalidated/reserved all grants and
/// will finish their publication before any interrupt or user context can run.
pub(in crate::user_tasks) fn publish(session: &mut Session, index: usize, arguments: [u64; 2]) {
    let draft = session.launches[index]
        .draft
        .take()
        .expect("missing completed draft");
    assert!(matches!(draft.phase, Phase::Ready));
    let child = session.launches[index].model.child;
    let task = &mut session.tasks[child];
    assert!(task.space.is_none());
    task.closing = Default::default();
    task.context = Context::user(draft.plan.entry, USER_STACK_TOP, arguments);
    task.report = Report {
        frames: draft.frames(),
        ..Default::default()
    };
    task.space = Some(draft.space);
    session.scheduler.admit(child);
}
