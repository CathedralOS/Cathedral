//! Progress independent storage and retain a completed read until Omega admits it.
use super::*;
impl Host {
    pub(super) fn progress_storage(&mut self) -> Result<(), Error> {
        let parent = &mut *self.parent;
        let server = &mut *self.server;
        let ready_launch = &mut self.ready_launch;
        #[cfg(feature = "interaction-delay")]
        let children = &mut *self.children;
        // The test-only probe can count this matrix's storage as busy, as a
        // slow provider would, around a close, so its answer is seen to wait.
        #[cfg(feature = "interaction-delay")]
        let hold_storage = {
            if core::mem::take(&mut parent.hold_storage_requested) {
                parent.hold_storage_armed = true;
                cathedral_user_runtime::time::storage_sample(
                    "interaction-delay-armed",
                    &[wire::STORAGE_BASE],
                );
            }
            let now = cathedral_user_runtime::time::now()?;
            let holding = parent
                .hold_storage_until
                .is_some_and(|until| !cathedral_user_runtime::time::reached(now, until));
            if parent.hold_storage_active && !holding {
                parent.hold_storage_until = None;
                cathedral_user_runtime::time::storage_sample("interaction-delay-released", &[]);
                children.release_held_loss();
            }
            parent.hold_storage_active = holding;
            holding
        };
        #[cfg(not(feature = "interaction-delay"))]
        let hold_storage = false;
        if !hold_storage && let Some(launched) = parent.progress_storage(server)? {
            assert!(ready_launch.is_none());
            *ready_launch = Some(launched);
        }
        self.hold_storage = hold_storage;
        Ok(())
    }
}
