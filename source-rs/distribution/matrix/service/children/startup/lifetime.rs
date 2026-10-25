//! Execute Omega's local startup recovery. Teardown finishes before new admission.
//! No ancestor chooses this matrix's picker or background, or gets their realm.
use super::*;
#[derive(Default)]
pub(in crate::service::children) struct Recovery {
    pub policy: cathedral_stock_matrix::policy::startup::State,
    /// The realm's picker and users folder, resolved once at startup.
    pub picker: Option<(u64, u64)>,
    /// The owner ran the picker itself and restarts it; this matrix only
    /// orders admission around it.
    pub owner_recovers: bool,
}
impl Children {
    pub(in crate::service::children) fn startup_retired(&mut self, slot: usize, failed: bool) {
        use cathedral_stock_matrix::policy::startup::{Context, Role};
        let role = match slot {
            0 => Role::Picker,
            1 => Role::User,
            BACKGROUND => Role::Background,
            _ => Role::Other,
        };
        if self.startup_recovery.policy.retired(
            role,
            failed,
            Context {
                login: self.login,
                entered: self.entered,
                desktop: self.desktop,
                owner_recovers: self.startup_recovery.owner_recovers,
            },
        ) {
            self.entered = false;
        }
    }

    pub(super) fn recover_startup(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        use cathedral_stock_matrix::policy::startup::Picker;
        let blocked = self.pending.iter().any(Option::is_some) || self.loss.is_some();
        match self.startup_recovery.policy.take_picker(blocked) {
            Picker::Wait => (),
            Picker::Exhausted => {
                // The owner sees a failed Nested app; another click is an explicit attempt.
                cathedral_user_runtime::write(
                    b"Cathedral nested desktop: picker retry exhausted\n",
                )?;
                return Err(Error(abi::IO_ERROR as i64));
            }
            Picker::Restart => {
                let (picker, users) = self
                    .startup_recovery
                    .picker
                    .ok_or(Error(abi::IO_ERROR as i64))?;
                self.start_picker(parent, world, server, picker, users)?;
                cathedral_user_runtime::write(
                    b"Cathedral nested desktop: local picker restored\n",
                )?;
            }
        }
        if self.startup_recovery.policy.take_background(blocked) {
            if let Err(error) = self.background(None, parent, world, server) {
                time::storage_sample("background-retry-refused", &[error.0 as u64]);
            }
            cathedral_user_runtime::write(b"Cathedral matrix: background retry consumed\n")?;
        }
        Ok(())
    }
}
