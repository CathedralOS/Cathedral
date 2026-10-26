// Omega sequences setup; the host keeps the saved record, image and local resources.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config;
use cathedral_platform_contracts::storage;
use cathedral_stock_matrix::policy::{
    self,
    realm::{Command, Job, State},
};
use cathedral_user_runtime::{memory::Object, storage::Scope};
impl Children {
    pub(super) fn configure(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        self.configure_object(None, parent, world, server)
    }
    pub(super) fn configure_object(
        &mut self,
        config: Option<u64>,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let mut state = State::new(
            Job::Configuration,
            self.desktop,
            self.slots.iter().any(Option::is_some) || parent.storing(),
            false,
        );
        let mut setup = workspace::Workspace::new(config);
        loop {
            let command = state.command().expect("completed realm setup step");
            if command == Command::Finish {
                assert!(state.complete(command, 0, 0));
                return if state.status() == 0 {
                    Ok(())
                } else {
                    Err(Error(state.status() as i64))
                };
            }
            let status =
                setup.execute(command, state.last_status(), self, parent, world, server)?;
            assert!(state.complete(command, status, 0));
        }
    }
}
fn rect([x, y, width, height]: [u64; 4]) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
