//! User-task session entrance: validate -> prepare -> admit -> execute -> collect.
//! Follow execution for the boot loop, traps for CPU re-entry, and syscalls for resource dispatch.
use super::{Config, Error, Exit, Program, Report, Supervision};
use crate::extent::FrameAllocator;
use alloc::vec::Vec;
use cathedral_arch as arch;
use core::sync::atomic::Ordering;
mod admission;
mod composition;
mod execution;
mod lifecycle;
mod links;
mod prepare;
mod state;
mod syscalls;
mod traps;
use state::{ACTIVE, Frames, LaunchState, LinkState, Session, Task, next_epoch};
use syscalls::{keyboard, tasks as taskcalls};
/// Run a bounded session, returning task outcomes only after complete reclamation.
/// frame_limit supports deterministic admission-failure testing.
/// # Safety
/// Sole CPU, IRQs off, no kernel/user task session; initialized heap/IDT/timer.
/// Kernel root, layout and image are live; frame reclamation is enabled. Output
/// is bounded, cannot allocate/suspend, and does not retain its borrowed buffer.
pub unsafe fn run(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    programs: &[Program<'_>],
    output: fn(&[u8]) -> bool,
    frame_limit: usize,
) -> Result<Vec<Report>, Error> {
    // SAFETY: Same session and memory obligations as the caller.
    unsafe {
        run_configured(
            frames,
            layout,
            image,
            programs,
            output,
            Config {
                links: &[],
                frame_limit,
                endpoints: &[],
                supervision: &[],
                clock_readers: &[],
            },
        )
    }
}

/// Run tasks with explicit boot-issued endpoint, launch and clock grants.
/// # Safety
/// Same obligations as run. Endpoint indices refer to initial programs; clock
/// indices may also name reserved child slots.
/// Any framebuffer must be a live, reserved device aperture exclusively held by
/// boot: no RAM allocation or other CPU/alias may access its pages. The launch
/// grant authorizes its executable to read/write the entire aperture for its lifetime.
/// A keyboard grant requires exclusive PC-controller/PIC IRQ1 custody. This
/// bootstrap profile has no concurrent firmware, mouse or other controller user.
pub unsafe fn run_configured(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    programs: &[Program<'_>],
    output: fn(&[u8]) -> bool,
    config: Config<'_>,
) -> Result<Vec<Report>, Error> {
    assert!(ACTIVE.load(Ordering::Acquire).is_null());
    let count = composition::validate(programs.len(), &config)?;
    let baseline = frames.allocated();
    let (mut session, mut reports) =
        prepare::session(programs.len(), count, &config, baseline, output)?;
    let mut source = Frames {
        frames,
        remaining: config.frame_limit,
    };
    // SAFETY: Caller supplies exclusive machine/memory custody. Initial admission
    // rolls back on failure; execution pins this session until all tasks are reaped.
    unsafe {
        admission::initial(&mut session, layout, image, programs, &mut source)?;
        execution::run(&mut session, &mut source, layout, image, &config);
    }
    for task in &mut session.tasks {
        reports.push(core::mem::take(&mut task.report));
    }
    assert_eq!(source.frames.allocated(), baseline);
    Ok(reports)
}
