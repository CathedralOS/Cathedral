//! F5 copied-image faults: arm a real provider only after a visible dirty note.
//! The normal owner watchdog handles every timeout and replacement afterward.
use super::*;
use core::sync::atomic::{AtomicU64, Ordering};

static DISABLED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
pub(super) fn disable() {
    DISABLED.store(true, Ordering::Relaxed);
}
pub(super) fn observe(session: &mut admission::Session) {
    if DISABLED.load(Ordering::Relaxed) {
        return;
    }
    static ARMED: AtomicU64 = AtomicU64::new(0);
    if session.user == 0 || session.storage_watch.pending.is_some() {
        return;
    }
    let Some(provider) = session.storage.as_mut() else {
        return;
    };
    if provider.starting.is_some() {
        return;
    }
    let previous = ARMED.load(Ordering::Relaxed);
    if previous == provider.child.raw() || (previous != 0 && !cfg!(feature = "storage-live-repeat"))
    {
        return;
    }
    let y = if cfg!(feature = "storage-live-deep") {
        98
    } else if cfg!(feature = "storage-live-nested") {
        66
    } else {
        34
    };
    if call(session.matrix, [wire::READ_PIXEL, 237, y, 0, 0, 0, 0, 0])[..2] != [0, 0x9b7028] {
        return;
    }
    let phase = if cfg!(feature = "storage-live-postcommit") {
        2
    } else {
        1
    };
    let (bytes, len) = provider.command(&[0xf5, phase, storage::PUBLISH as u8, 0]);
    assert_eq!(&bytes[..len], b"armed");
    ARMED.store(provider.child.raw(), Ordering::Relaxed);
    write(b"Cathedral desktop: live storage probe armed\n").unwrap();
}
