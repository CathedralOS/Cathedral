// Isolated-task entrance: prepare authority and storage, reserve RAM, admit, run, collect.
// Startup owns composition; admission owns images; execution owns the boot/trap loop.
// Follow syscalls for requests, lifecycle for death/restart, memory for live page objects.
use crate::memory::FrameAllocator;
use alloc::vec::Vec;
use cathedral_arch as arch;
use core::sync::atomic::Ordering;
use memory::frames::Frames;
use session::{ACTIVE, LaunchState, LinkState, Session, Task, next_epoch};
use syscalls::{devices::keyboard, tasks as taskcalls};
/// Run tasks with explicit boot-issued endpoint, launch and clock grants.
/// # Safety
/// Same obligations as run. Endpoint indices refer to initial programs; clock
/// indices may also name reserved child slots.
/// Any framebuffer must be a live, reserved device aperture exclusively held by
/// boot: no RAM allocation or other CPU/alias may access its pages. The launch
/// grant authorizes its executable to read/write the entire aperture for its lifetime.
/// A keyboard grant requires exclusive PC-controller/PIC IRQ1 custody. This
/// bootstrap profile has no concurrent firmware, mouse or other controller user.
/// Each register-device grant requires the corresponding board device to exist,
/// with exclusive register and IRQ custody for this entire session (including
/// reset). The PC auxiliary serial device is COM2/IRQ3, separate from diagnostics.
pub unsafe fn run_configured(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    programs: &[Program<'_>],
    output: fn(&[u8]) -> bool,
    config: Config<'_>,
) -> Result<Vec<Report>, Error> {
    assert!(ACTIVE.load(Ordering::Acquire).is_null());
    let baseline = frames.allocated();
    let (mut session, mut reports) = startup::prepare(programs.len(), &config, baseline, output)?;
    let mut source = Frames::new(
        frames,
        session.memory.funding.capacity(),
        config.frame_limit,
    )?;
    // SAFETY: Caller supplies exclusive machine/memory custody. Initial admission
    // rolls back on failure; execution pins this session until all tasks are reaped.
    unsafe {
        admission::initial(&mut session, layout, image, programs, &mut source)?;
        execution::run(&mut session, &mut source, layout, image, &config);
    }
    for (slot, task) in session.tasks.iter_mut().enumerate() {
        task.report.cpu = session.scheduler.cpu_report(slot);
        reports.push(core::mem::take(&mut task.report));
    }
    assert_eq!(source.allocated(), baseline);
    Ok(reports)
}

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
                executables: &[],
                dynamic_tasks: 0,
                links: &[],
                frame_limit,
                endpoints: &[],
                supervision: &[],
                memory: &[],
                channel_limits: &[],
                cpu_limits: &[],
                clock_readers: &[],
            },
        )
    }
}
