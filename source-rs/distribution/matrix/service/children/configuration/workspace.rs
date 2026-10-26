// Execute one configuration operation. No connection, image or task enters Omega.
use super::*;
use cathedral_stock_matrix::policy::resources;
pub(super) struct Workspace {
    config: Option<u64>,
    saved: Option<config::Config>,
    launcher: u64,
    background: Option<u64>,
    code: Option<Object>,
    rects: [[u64; 4]; 2],
    plan: Option<parent::LaunchRequest>,
}
impl Workspace {
    pub(super) fn new(config: Option<u64>) -> Self {
        Self {
            config,
            saved: None,
            launcher: 0,
            background: None,
            code: None,
            rects: [[0; 4]; 2],
            plan: None,
        }
    }
    pub(super) fn execute(
        &mut self,
        command: Command,
        last: u64,
        children: &mut Children,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<u64, Error> {
        use Command::*;
        match command {
            Refresh => {
                #[cfg(feature = "startup-fault")]
                if children.slots.iter().all(Option::is_none) {
                    panic!("deliberate matrix startup failure");
                }
                parent.refresh(world)?;
            }
            ReadConfiguration => self.read(parent)?,
            ValidateConfiguration => return self.validate(parent),
            AdmitLauncher => {
                let storage = children.run_object(
                    self.plan.expect("validated plan"),
                    self.code.as_mut().expect("retained launcher image"),
                    parent,
                    server,
                    world,
                )?;
                return Ok(if storage { 0 } else { abi::DENIED });
            }
            ReleaseImage => {
                self.code.take();
            }
            BindLauncher => return Ok(parent.storage_grant(server, 0, 0, 0)?.err().unwrap_or(0)),
            InstallWorkspace => self.install(children),
            PlaceLauncher => {
                let saved = self.saved.expect("configuration");
                let offer = resources::launcher(saved.launcher_pages);
                return Ok(world
                    .surface_range(
                        0,
                        world
                            .backed_surfaces()
                            .saturating_sub(offer.surfaces as usize),
                        offer.surfaces as usize,
                    )
                    .and_then(|()| world.place(0, rect(self.rects[0]), saved.keys))
                    .err()
                    .unwrap_or(0));
            }
            StartBackground => {
                return Ok(children
                    .background(self.background, parent, world, server)
                    .err()
                    .map_or(0, |e| e.0 as u64));
            }
            BackgroundFailed => {
                time::storage_sample("background-start-refused", &[last]);
                children.startup_recovery.policy.background_failed();
            }
            Flush => super::super::super::requests::flush(parent, world)?,
            Notify => children.notify(world)?,
            GrantChrome => {
                parent.grant_recovery(0, server.incarnation(0));
                world.grant_layout(0, server.incarnation(0).expect("configured launcher"));
                children
                    .management
                    .grant(0, server.incarnation(0).unwrap(), 1)
                    .map_err(|c| Error(c as i64))?;
            }
            RecordLauncher => {
                children.launcher = Some(recovery::Launcher::new(
                    self.plan.expect("launcher plan"),
                    children.slots[0].as_ref().unwrap().task.raw(),
                    server.incarnation(0).unwrap(),
                ));
                cathedral_user_runtime::write(b"Cathedral matrix: configured from realm paths\n")?;
            }
            ClearWorkspace => {
                children.desktop = false;
                children.standing = [None; CHILDREN];
            }
            StopLauncher => children.stop(0, server, world, parent)?,
            ResetSurfaces => surface_limit(world, 1, 0).map_err(|c| Error(c as i64))?,
            _ => return Err(Error(abi::IO_ERROR as i64)),
        }
        Ok(0)
    }
    fn read(&mut self, parent: &mut Parent) -> Result<(), Error> {
        let (saved, launcher, background, code) = parent.local_storage(|scope| {
            scope.bind().map_err(|c| Error(c as i64))?;
            let object = match self.config {
                Some(object) => object,
                None => scope.lookup(config::NAME).map_err(|c| Error(c as i64))?,
            };
            let mut bytes = read(scope, object, Some(config::SIZE))?;
            let saved = policy::decode(&bytes.map()?[..config::SIZE])
                .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
            drop(bytes);
            let mut ids = [None; 2];
            scope
                .named(
                    &[storage::metadata::text(&saved.launcher), b"Background"],
                    &mut ids,
                )
                .map_err(|c| Error(c as i64))?;
            let launcher = ids[0].ok_or(Error(storage::MISSING as i64))?;
            let code = read(scope, launcher, None)?;
            Ok((saved, launcher, ids[1], code))
        })?;
        self.saved = Some(saved);
        self.launcher = launcher;
        self.background = background;
        self.code = Some(code);
        Ok(())
    }
    fn validate(&mut self, parent: &Parent) -> Result<u64, Error> {
        let saved = self.saved.expect("read configuration");
        if saved.keys & !parent.grant.keys != 0 {
            return Ok(abi::DENIED);
        }
        let Some(rects) = policy::rectangles(
            saved.margin,
            saved.launcher_height,
            saved.gap,
            parent.grant.width,
            parent.grant.height,
        ) else {
            return Ok(abi::INVALID_ARGUMENT);
        };
        self.rects = rects;
        let offer = resources::launcher(saved.launcher_pages);
        self.plan = Some(parent::LaunchRequest {
            slot: 0,
            object: self.launcher,
            scope: 0,
            argument: wire::launch_argument(0, self.launcher),
            channels: offer.channels,
            descendants: offer.descendants,
            cpu: offer.cpu,
            pages: Some(offer.pages),
            stamp: 0,
            imports: offer.imports,
        });
        Ok(0)
    }
    fn install(&self, children: &mut Children) {
        let saved = self.saved.expect("configuration");
        let offer = resources::workspace([saved.app_pages, saved.app_readers], saved.app_surfaces);
        children.saved_storage[0] = Some((0, 0));
        children.desktop = true;
        for slot in 1..BACKGROUND {
            #[cfg(feature = "input-producer-probe")]
            {
                children.allowed_imports[slot] = Some(
                    cathedral_platform_contracts::imports::ALL
                        | cathedral_platform_contracts::imports::INPUT_PRODUCER,
                );
            }
            children.standing[slot] = Some(Standing {
                pages: offer.pages,
                channels: offer.channels,
                descendants: offer.descendants,
                cpu: offer.cpu,
                surfaces: offer.surfaces,
                place: Some((rect(self.rects[1]), saved.keys)),
            });
        }
    }
}
