//! Address-space construction and task installation use separate stack frames.
//! Slots are allocated before execution; live admission never grows the arena.
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
    let task = &mut session.tasks[slot];
    assert!(task.space.is_none());
    // SAFETY: Caller serializes admission on the boot stack/kernel root with IRQs
    // off. Vacant slot cannot run; load rolls back its own partial mappings.
    let entry = unsafe { load(&mut task.space, layout, image, program, frames)? };
    task.context = Context::user(entry, arch::USER_STACK_TOP, program.arguments);
    task.receive = None;
    task.wait = None;
    task.report = Report {
        frames: task.space.as_ref().unwrap().frame_count(),
        ..Default::default()
    };
    session.scheduler.admit(slot);
    Ok(())
}

unsafe fn load(
    destination: &mut Option<UserSpace>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    frames: &mut Frames<'_>,
) -> Result<u64, Error> {
    let entry;
    // SAFETY: Same serialized mapping obligations as admit. Both constructors
    // retain custody until success and release partial allocations on failure.
    let result = unsafe {
        match &program.executable {
            Executable::Probe(code) => {
                entry = arch::USER_CODE;
                UserSpace::create(layout, image, code, frames)
            }
            Executable::Elf(bytes) => {
                let executable = elf::parse(bytes).map_err(Error::Executable)?;
                entry = executable.entry;
                UserSpace::from_segments(layout, image, executable.segments(), frames)
            }
        }
    };
    match result {
        Ok(space) => {
            *destination = Some(space);
            Ok(entry)
        }
        Err(error) => Err(Error::Memory(error)),
    }
}

pub(super) unsafe fn retire(task: &mut Task, frames: &mut Frames<'_>) {
    // SAFETY: Caller retired this task or has not yet published any context.
    unsafe {
        task.space.take().unwrap().release(frames);
    }
}

pub(super) fn reserve_slot(session: &mut Session) {
    session.tasks.push(Task {
        context: Context::default(),
        space: None,
        report: Report::default(),
        receive: None,
        wait: None,
    });
}
