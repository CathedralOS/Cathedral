// Boot-stack execution loop: reap and admit before resuming a user context.
use super::{ACTIVE, Config, Frames, Session, arch, lifecycle};
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
            deferred::step(&mut *session, source, layout, image, config);
            if (*session).scheduler.finished() && !deferred::pending(&*session) {
                break;
            }
            // The step has returned: no exclusive session borrow or partial
            // publication survives this interrupt window / context suspension.
            if deferred::pending(&*session) {
                arch::service_pending_interrupts();
            }
            (*session).scheduler.account_cpu(arch::cpu_time_us());
            if (*session).scheduler.has_eligible_ready() || deferred::pending(&*session) {
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
