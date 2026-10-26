// Background setup/retry uses the same Omega sequence and local-only resource custody.
use super::*;
impl Children {
    pub(in crate::service::children) fn background(
        &mut self,
        known: Option<u64>,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let mut state = State::new(Job::Background, false, false, false);
        let mut code: Option<Object> = None;
        let mut object = 0;
        loop {
            let command = state.command().expect("completed background step");
            let mut status = 0;
            match command {
                Command::ReadBackground => {
                    let read = parent.local_storage(|scope| {
                        scope.bind().map_err(|c| Error(c as i64))?;
                        let object = match known {
                            Some(object) => object,
                            None => scope.lookup(b"Background").map_err(|c| Error(c as i64))?,
                        };
                        Ok((object, read(scope, object, None)?))
                    })?;
                    object = read.0;
                    code = Some(read.1);
                }
                Command::AdmitBackground => {
                    let plan = parent::LaunchRequest {
                        slot: BACKGROUND,
                        object,
                        scope: 0,
                        argument: wire::launch_argument(0, object),
                        channels: 2,
                        descendants: 0,
                        cpu: 0,
                        pages: Some([5, 2]),
                        stamp: 0,
                        imports: cathedral_platform_contracts::imports::INTERACTION,
                    };
                    // The returned boolean reports storage acquisition, not admission.
                    self.run_object(
                        plan,
                        code.as_mut().expect("background image"),
                        parent,
                        server,
                        world,
                    )?;
                }
                Command::PlaceBackground => {
                    status = world
                        .surface_range(BACKGROUND, 0, 1)
                        .and_then(|()| {
                            world.place(
                                BACKGROUND,
                                Rect {
                                    x: 0,
                                    y: 0,
                                    width: parent.grant.width,
                                    height: parent.grant.height,
                                },
                                0,
                            )
                        })
                        .err()
                        .unwrap_or(0);
                }
                Command::StopBackground => self.stop(BACKGROUND, server, world, parent)?,
                Command::RecordBackground => cathedral_user_runtime::write(
                    b"Cathedral matrix: background admitted with presentation only\n",
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
