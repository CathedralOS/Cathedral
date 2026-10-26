// Execute Omega's picker setup while retaining the image and scoped user binding locally.
use super::*;
impl Children {
    pub(in crate::service::children) fn start_picker(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
        picker: u64,
        users: u64,
    ) -> Result<(), Error> {
        let mut state = State::new(Job::Picker, false, false, false);
        let mut code = None;
        let mut picker_mode = 0;
        let mut capacity = 0;
        loop {
            let command = state.command().expect("completed picker setup step");
            let mut status = 0;
            match command {
                Command::PreparePicker => {
                    picker_mode = self.startup_recovery.policy.picker_started();
                    self.startup_recovery.picker = Some((picker, users));
                }
                Command::ReadPicker => {
                    code = Some(parent.local_storage(|scope| {
                        scope.bind().map_err(|c| Error(c as i64))?;
                        configuration::read(scope, picker, None)
                    })?);
                }
                Command::ValidatePicker => {
                    capacity = world.backed_surfaces();
                    if capacity < wire::config::HOST_SURFACES {
                        status = abi::NO_MEMORY;
                    }
                }
                Command::AdmitPicker => {
                    self.run_object(
                        parent::LaunchRequest {
                            slot: 0,
                            object: picker,
                            scope: users,
                            argument: wire::launch_argument(picker_mode, picker),
                            channels: 3,
                            descendants: 0,
                            cpu: 0,
                            pages: Some([48, wire::CHOOSER_READERS]),
                            stamp: 0,
                            imports: cathedral_platform_contracts::imports::ALL,
                        },
                        code.as_mut().expect("picker image"),
                        parent,
                        server,
                        world,
                    )?;
                }
                Command::BindPicker => {
                    status = parent
                        .storage_grant(server, 0, 0, users)?
                        .err()
                        .unwrap_or(0);
                    if status == 0 {
                        self.saved_storage[0] = Some((0, users));
                    }
                }
                Command::PlacePicker => {
                    world
                        .surface_range(0, capacity - 6, 6)
                        .map_err(|c| Error(c as i64))?;
                    world
                        .place(
                            0,
                            Rect {
                                x: 0,
                                y: 0,
                                width: parent.grant.width,
                                height: parent.grant.height,
                            },
                            parent.grant.keys,
                        )
                        .map_err(|c| Error(c as i64))?;
                }
                Command::InstallPicker => {
                    self.standing[1] = Some(Standing {
                        pages: [256, 384],
                        channels: 24,
                        descendants: 3,
                        cpu: 0,
                        surfaces: (capacity - 6) as u64,
                        place: Some((
                            Rect {
                                x: 0,
                                y: 0,
                                width: parent.grant.width,
                                height: parent.grant.height,
                            },
                            parent.grant.keys,
                        )),
                    });
                    self.login = true;
                }
                Command::Notify => self.notify(world)?,
                Command::RecordPicker => cathedral_user_runtime::write(
                    b"Cathedral nested desktop: scoped picker ready\n",
                )?,
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
            assert!(state.complete(command, status, 0));
        }
    }
}
