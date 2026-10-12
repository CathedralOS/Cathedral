// Address-space construction and task installation use separate stack frames.
// Slots are allocated before execution; live admission never grows the arena.
use super::{Error, Frames, Report, Session, Task};
use crate::user_tasks::address_space::{USER_CODE, USER_STACK_TOP, UserSpace};
use crate::user_tasks::{Executable, Program};
use cathedral_arch::{self as arch, Context};

pub(super) unsafe fn admit(
    session: &mut Session,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    frames: &mut Frames<'_>,
    slot: usize,
) -> Result<(), Error> {
    let allowance = session.memory.funding.available(slot);
    let previous = frames.remaining;
    frames.remaining = previous.min(allowance);
    let baseline = frames.allocated();
    // SAFETY: Same inactive-root and sole-CPU obligations; all failures roll back.
    let result = unsafe { construct(session, layout, image, program, frames, slot) };
    let consumed = previous.min(allowance) - frames.remaining;
    frames.remaining = previous - consumed;
    if result.is_ok() {
        session
            .memory
            .funding
            .charge(slot, frames.allocated() - baseline)
            .expect("serialized admission exceeded reserved funding");
    } else {
        assert_eq!(frames.allocated(), baseline);
    }
    result
}

unsafe fn construct(
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
    if session.memory.model.enabled(slot) {
        // SAFETY: Serialized admission; reserve tables before publishing the new user root.
        if let Err(error) = unsafe { task.space.as_mut().unwrap().prepare_regions(frames) } {
            // SAFETY: No context has been published; partial table allocation belongs to this task.
            unsafe {
                discard(task, frames);
            }
            return Err(Error::Memory(error));
        }
    }
    if let Some(fb) = session
        .launches
        .iter()
        .find(|launch| launch.model.child == slot)
        .and_then(|launch| launch.framebuffer)
    {
        // SAFETY: Boot bound this reserved aperture exclusively to the child.
        // Space is inactive; failed device-table allocation rolls back all RAM.
        if let Err(error) = unsafe { task.space.as_mut().unwrap().map_framebuffer(fb, frames) } {
            // SAFETY: Admission failed before publishing the new root/context.
            unsafe {
                discard(task, frames);
            }
            return Err(Error::Memory(error));
        }
    }
    task.context = Context::user(entry, USER_STACK_TOP, program.arguments);
    task.receive = None;
    task.wait = None;
    task.keyboard_wait = false;
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
                entry = USER_CODE;
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

pub(super) unsafe fn retire(session: &mut Session, slot: usize, frames: &mut Frames<'_>) {
    let count = session.tasks[slot].space.as_ref().unwrap().frame_count();
    // SAFETY: Caller has removed all mappings/borrowers and retired the context.
    unsafe {
        discard(&mut session.tasks[slot], frames);
    }
    session.memory.funding.refund(slot, count);
}
unsafe fn discard(task: &mut Task, frames: &mut Frames<'_>) {
    // SAFETY: Caller retired this task or has not yet published any context.
    unsafe {
        task.space.take().unwrap().release(frames);
    }
}

pub(super) fn reserve_slot(session: &mut Session) {
    session.tasks.push(Task {
        memory_call: None,
        context: Context::default(),
        space: None,
        report: Report::default(),
        receive: None,
        wait: None,
        notify: None,
        keyboard_wait: false,
        keyboard_deadline: None,
    });
}

/// Admit the initial set atomically; retire every earlier task on failure.
pub(super) unsafe fn initial(
    session: &mut Session,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    programs: &[Program<'_>],
    source: &mut Frames<'_>,
) -> Result<(), Error> {
    for (slot, program) in programs.iter().enumerate() {
        // Keep large address-space construction temporaries out of the session frame.
        if let Err(error) =
            // SAFETY: Serialized admission on kernel CR3; no user has started.
            unsafe { admit(session, layout, image, program, source, slot) }
        {
            for slot in 0..session.tasks.len() {
                if session.tasks[slot].space.is_none() {
                    continue;
                }
                // SAFETY: Admission failed before publishing any context/root.
                unsafe {
                    retire(session, slot, source);
                }
            }
            assert_eq!(source.allocated(), session.frame_baseline);
            return Err(error);
        }
    }
    Ok(())
}
