//! Local stock-process retry policy. Teardown finishes before any new admission.
//! No ancestor chooses this matrix's picker or background, or gets their realm.
use super::*;
#[derive(Default)]
pub(in crate::service::children) struct Recovery {
    pub picker_attempts: u8,
    pub picker_pending: bool,
    pub selection_failed: bool,
    pub background_pending: bool,
    pub background_retried: bool,
    /// The realm's picker and users folder, resolved once at startup.
    pub picker: Option<(u64, u64)>,
    /// The owner ran the picker itself and restarts it; this matrix only
    /// orders admission around it.
    pub owner_recovers: bool,
}
impl Children {
    pub(in crate::service::children) fn startup_retired(&mut self, slot: usize, failed: bool) {
        if !failed || (self.login && self.startup_recovery.owner_recovers) {
            return;
        }
        if self.login && slot == 1 && self.entered {
            self.entered = false;
            self.startup_recovery.selection_failed = true;
            self.startup_recovery.picker_attempts = 0;
            self.startup_recovery.picker_pending = true;
        } else if self.login && slot == 0 && !self.entered {
            self.startup_recovery.picker_pending = true;
        } else if self.desktop && slot == BACKGROUND && !self.startup_recovery.background_retried {
            self.startup_recovery.background_pending = true;
        }
    }
    pub(super) fn recover_startup(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.pending.iter().any(Option::is_some) || self.loss.is_some() {
            return Ok(());
        }
        if self.startup_recovery.picker_pending {
            self.startup_recovery.picker_pending = false;
            if self.startup_recovery.picker_attempts >= 2 {
                // The outer taskbar reports a failed Nested app. No retry loop
                // and no blank immortal host: a new click is an explicit attempt.
                cathedral_user_runtime::write(
                    b"Cathedral nested desktop: picker retry exhausted\n",
                )?;
                return Err(Error(abi::IO_ERROR as i64));
            }
            let (picker, users) = self
                .startup_recovery
                .picker
                .ok_or(Error(abi::IO_ERROR as i64))?;
            self.start_picker(parent, world, server, picker, users)?;
            cathedral_user_runtime::write(b"Cathedral nested desktop: local picker restored\n")?;
        }
        if self.startup_recovery.background_pending {
            self.startup_recovery.background_pending = false;
            self.startup_recovery.background_retried = true;
            if let Err(error) = self.background(parent, world, server) {
                time::storage_sample("background-retry-refused", &[error.0 as u64]);
            }
            cathedral_user_runtime::write(b"Cathedral matrix: background retry consumed\n")?;
        }
        Ok(())
    }
}
