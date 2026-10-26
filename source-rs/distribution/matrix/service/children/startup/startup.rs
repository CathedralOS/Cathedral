// Omega chooses the realm shape and setup sequence; this host reads only its supplied realm.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config;
use cathedral_stock_matrix::policy::realm::{Command, Job, State};
impl Children {
    pub(in crate::service) fn startup(
        &mut self,
        mode: &mut Startup,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let mut state = State::new(Job::Discovery, false, false, *mode == Startup::Realm);
        let mut entries = [None; 3];
        loop {
            let command = state.command().expect("completed realm discovery step");
            let mut status = 0;
            let mut detail = 0;
            match command {
                Command::Refresh => {
                    parent.refresh(world)?;
                    detail = u64::from(parent.grant.width != 0);
                }
                Command::InspectRealm => match self.inspect_realm(parent) {
                    Ok(ids) => {
                        entries = ids;
                        for (i, id) in ids.iter().enumerate() {
                            detail |= u64::from(id.is_some()) << i;
                        }
                    }
                    Err(error) => status = error.0 as u64,
                },
                Command::ConfigureRealm => {
                    self.configure_object(entries[0], parent, world, server)?
                }
                Command::StartPicker => self.start_picker(
                    parent,
                    world,
                    server,
                    entries[1].expect("realm Login"),
                    entries[2].expect("realm Users"),
                )?,
                Command::MarkHosted => *mode = Startup::Hosted,
                Command::RecoverStartup => self.recover_startup(parent, world, server)?,
                Command::Finish => {
                    assert!(state.complete(command, 0, 0));
                    return if state.status() == 0 {
                        Ok(())
                    } else {
                        Err(Error(state.status() as i64))
                    };
                }
                _ => return Err(Error(abi::IO_ERROR as i64)),
            }
            assert!(state.complete(command, status, detail));
        }
    }
    fn inspect_realm(&self, parent: &mut Parent) -> Result<[Option<u64>; 3], Error> {
        parent.local_storage(|scope| {
            // No binding leaves setup to the owner, as in the root/headless adapters.
            if scope.bind().is_err() {
                return Ok([None; 3]);
            }
            let mut ids = [None; 3];
            scope
                .named(&[config::NAME, b"Login", b"Users"], &mut ids)
                .map_err(|c| Error(c as i64))?;
            Ok(ids)
        })
    }
}
