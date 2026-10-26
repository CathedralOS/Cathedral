// Execute Omega's picker setup while retaining the image and scoped user binding locally.
use super::*;
use cathedral_stock_matrix::policy::resources;
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
        let mut offer = None;
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
                    match resources::picker(capacity) {
                        Ok(value) => offer = Some(value),
                        Err(code) => status = code,
                    }
                }
                Command::AdmitPicker => {
                    let offer = offer.expect("validated picker offer");
                    self.run_object(
                        parent::LaunchRequest {
                            slot: 0,
                            object: picker,
                            scope: users,
                            argument: wire::launch_argument(picker_mode, picker),
                            channels: offer.channels,
                            descendants: offer.descendants,
                            cpu: offer.cpu,
                            pages: Some(offer.pages),
                            stamp: 0,
                            imports: offer.imports,
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
                    let surfaces = offer.expect("validated picker offer").surfaces as usize;
                    world
                        .surface_range(0, capacity - surfaces, surfaces)
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
                    let offer = resources::user(capacity).map_err(|c| Error(c as i64))?;
                    self.standing[1] = Some(Standing {
                        pages: offer.pages,
                        channels: offer.channels,
                        descendants: offer.descendants,
                        cpu: offer.cpu,
                        surfaces: offer.surfaces,
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
