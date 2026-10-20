// Boot-stack work entrance: retire authority/backing first, admit one child,
// then advance one memory call. No session borrow survives the caller's yield.
use super::{Config, Frames, Session, arch, lifecycle};
use crate::user_tasks::{Executable, Program, admission::pool::Code};

pub(super) unsafe fn step(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    config: &Config<'_>,
) {
    // SAFETY: Boot stack, kernel root, exclusive session and IRQs off throughout.
    unsafe {
        lifecycle::reap(session, source);
        if let Some(index) = session.deferred.next(Work::Admission) {
            let mut pages = [0; cathedral_kernel_contracts::memory::MAX_PAGES];
            let program = match session.launches[index]
                .code
                .expect("queued launch lacks code")
            {
                Code::Template(template) => {
                    if template < config.supervision.len() {
                        &config.supervision[template].program
                    } else {
                        &config.executables[template - config.supervision.len()]
                    }
                }
                Code::Object { region, role } => {
                    let frames = session.memory.frames_of(region);
                    pages[..frames.len()].copy_from_slice(frames);
                    &Program {
                        executable: Executable::Object(&pages[..frames.len()]),
                        arguments: [role, 0],
                    }
                }
                // Spawn reports refusal when the owner retries consumed code.
                Code::Admitted => &Program {
                    executable: Executable::Probe(&[]),
                    arguments: [0, 0],
                },
            };
            lifecycle::spawn(session, source, layout, image, program, index);
        }
        crate::user_tasks::memory::process(session, source);
    }
}

pub(super) fn pending(session: &Session) -> bool {
    session.deferred.any()
        || session.ipc.memory_retirement_pending()
        || session.memory.retirement_pending()
}

/// A deliberate boot-stack yield may dispatch a user even with pending work.
/// Its next trap returns to boot; asynchronous idle IRQs must not skip the work.
pub(super) fn return_to_boot(pending: bool, from_user: bool, explicit_yield: bool) -> bool {
    pending && (from_user || !explicit_yield)
}
