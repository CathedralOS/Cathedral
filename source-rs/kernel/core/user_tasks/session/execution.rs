//! Boot-stack execution loop: reap and admit before resuming a user context.
use super::{ACTIVE, Config, Frames, Session, arch, lifecycle, traps};
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
        arch::keyboard_irq(config.supervision.iter().any(|grant| grant.keyboard));
        loop {
            lifecycle::reap(&mut *session, source);
            for index in 0..(*session).launches.len() {
                if let Some(template) = (&(*session).launches)[index].template {
                    lifecycle::spawn(
                        &mut *session,
                        source,
                        layout,
                        image,
                        if template < config.supervision.len() {
                            &config.supervision[template].program
                        } else {
                            &config.executables[template - config.supervision.len()]
                        },
                        index,
                    );
                }
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
        arch::keyboard_irq(false);
        arch::set_switch_handler(None);
        arch::end_user_session();
        ACTIVE.store(core::ptr::null_mut(), Ordering::Release);
    }
}
