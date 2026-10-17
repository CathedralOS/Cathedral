// Boot-stack execution loop: reap and admit before resuming a user context.
use super::{ACTIVE, Config, Frames, Session, admission::pool::Code, arch, lifecycle};
use core::sync::atomic::Ordering;
/// # Safety
/// Initialized user entry machinery; exclusive pinned session, boot stack, IRQs off.
pub(super) unsafe fn run(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    config: &Config<'_>,
) {
    let session = &raw mut *session;
    ACTIVE.store(session, Ordering::Release);
    // SAFETY: Session remains pinned on mapped boot stack. Its vectors never grow;
    // callbacks get exclusive access only across suspension, never a live borrow.
    unsafe {
        arch::begin_user_session();
        arch::set_switch_handler(Some(traps::schedule));
        if let Some(keyboard) = arch::board::devices().keyboard {
            keyboard.interrupts(config.supervision.iter().any(|grant| grant.keyboard));
        }
        loop {
            lifecycle::reap(&mut *session, source);
            for index in 0..(*session).launches.len() {
                let mut pages = [0; cathedral_kernel_contracts::memory::MAX_PAGES];
                let program = match (&(*session).launches)[index].code {
                    None => continue,
                    Some(Code::Template(template)) => {
                        if template < config.supervision.len() {
                            &config.supervision[template].program
                        } else {
                            &config.executables[template - config.supervision.len()]
                        }
                    }
                    Some(Code::Object { region, role }) => {
                        let frames = (*session).memory.frames_of(region);
                        pages[..frames.len()].copy_from_slice(frames);
                        &super::Program {
                            executable: super::Executable::Object(&pages[..frames.len()]),
                            arguments: [role, 0],
                        }
                    }
                    // Spawn reports the refusal itself when the owner retries.
                    Some(Code::Admitted) => &super::Program {
                        executable: super::Executable::Probe(&[]),
                        arguments: [0, 0],
                    },
                };
                lifecycle::spawn(&mut *session, source, layout, image, program, index);
            }
            super::memory::process(&mut *session, source);
            if (*session).scheduler.finished() {
                break;
            }
            (*session).scheduler.account_cpu(arch::cpu_time_us());
            if (*session).scheduler.has_eligible_ready() {
                arch::suspend();
            } else {
                arch::wait_for_ticks(1);
            }
        }
        if let Some(keyboard) = arch::board::devices().keyboard {
            keyboard.interrupts(false);
        }
        arch::set_switch_handler(None);
        arch::end_user_session();
        ACTIVE.store(core::ptr::null_mut(), Ordering::Release);
    }
}
