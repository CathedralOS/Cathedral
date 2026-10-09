//! Admission has its own frame so debug construction copies do not accumulate
//! alongside the long-lived session and reclamation temporaries on the boot stack.
use super::{Error, Frames, Report, Session, Task};
use crate::users::{Executable, Program, elf};
use cathedral_arch::{self as arch, Context, UserSpace};

pub(super) unsafe fn admit(
    session: &mut Session,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    frames: &mut Frames<'_>,
    slot: usize,
) -> Result<(), Error> {
    // SAFETY: Sole CPU, kernel root, IRQs off; no task has run. Constructors roll
    // back their own partial allocations; caller retires earlier admitted tasks.
    let (space, entry) = unsafe {
        match &program.executable {
            Executable::Probe(code) => (
                UserSpace::create(layout, image, code, frames).map_err(Error::Memory)?,
                arch::USER_CODE,
            ),
            Executable::Elf(bytes) => {
                let executable = elf::parse(bytes).map_err(Error::Executable)?;
                (
                    UserSpace::from_segments(layout, image, executable.segments(), frames)
                        .map_err(Error::Memory)?,
                    executable.entry,
                )
            }
        }
    };
    let count = space.frame_count();
    session.tasks.push(Task {
        context: Context::user(entry, arch::USER_STACK_TOP, program.arguments),
        space: Some(space),
        receive: None,
        report: Report {
            frames: count,
            ..Default::default()
        },
    });
    session.scheduler.admit(slot);
    Ok(())
}

/// Keep by-value release temporaries out of the long-lived session frame.
pub(super) unsafe fn retire(task: &mut Task, frames: &mut Frames<'_>) {
    // SAFETY: Caller has retired this task or has not yet published any context.
    unsafe {
        task.space.take().unwrap().release(frames);
    }
}
