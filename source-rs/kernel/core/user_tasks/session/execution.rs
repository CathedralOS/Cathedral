//! Boot-stack execution loop: reap and admit before resuming a user context.
use super::{ACTIVE, Config, Frames, Session, arch, lifecycle, traps};
use crate::scheduler::TaskState;
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
            for (index, launch) in config.supervision.iter().enumerate() {
                lifecycle::spawn(&mut *session, source, layout, image, launch, index);
            }
            if (*session).scheduler.finished() {
                break;
            }
            if (*session).scheduler.states().contains(&TaskState::Ready) {
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
