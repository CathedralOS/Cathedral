//! Apply this matrix's saved arrangement to empty slots. The enclosing owner
//! triggers startup; it never chooses our internal launcher, placement or budgets.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config;
use cathedral_platform_contracts::storage;
use cathedral_stock_matrix::policy;
use cathedral_user_runtime::storage::Scope;

impl Children {
    /// An owner's CONFIGURE: the arrangement is found in the realm by name.
    pub(super) fn configure(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        self.configure_object(None, parent, world, server)
    }
    /// Apply the arrangement in `config`, or the realm's `Matrix` entry when
    /// the caller has not already found it.
    pub(super) fn configure_object(
        &mut self,
        config: Option<u64>,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        // A matrix that already read its arrangement from its realm is what
        // an owner's CONFIGURE asks for; the request is satisfied either way.
        if self.desktop {
            return Ok(());
        }
        if self.slots.iter().any(Option::is_some) || parent.storing() {
            return Err(Error(abi::BUSY as i64));
        }
        #[cfg(feature = "startup-fault")]
        if self.slots.iter().all(Option::is_none) {
            panic!("deliberate matrix startup failure");
        }
        parent.refresh(world)?;
        let (saved, launcher, background, mut code) = parent.local_storage(|scope| {
            scope.bind().map_err(|c| Error(c as i64))?;
            let object = match config {
                Some(object) => object,
                None => scope.lookup(config::NAME).map_err(|c| Error(c as i64))?,
            };
            let mut bytes = read(scope, object, Some(config::SIZE))?;
            let saved = policy::decode(&bytes.map()?[..config::SIZE])
                .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
            drop(bytes);
            // One listing names the launcher and the background together.
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
        if saved.keys & !parent.grant.keys != 0 {
            return Err(Error(abi::DENIED as i64));
        }
        let rects = policy::rectangles(
            saved.margin,
            saved.launcher_height,
            saved.gap,
            parent.grant.width,
            parent.grant.height,
        )
        .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let plan = parent::LaunchRequest {
            slot: 0,
            object: launcher,
            scope: 0,
            argument: cathedral_desktop_matrix_contracts::matrix::launch_argument(0, launcher),
            channels: 2,
            descendants: 0,
            cpu: 0,
            pages: Some([
                saved.launcher_pages,
                cathedral_desktop_matrix_contracts::matrix::CHOOSER_READERS,
            ]),
            stamp: 0,
            imports: cathedral_platform_contracts::imports::ALL,
        };
        if !self.run_object(plan, &mut code, parent, server, world)? {
            self.stop(0, server, world, parent)?;
            return Err(Error(abi::DENIED as i64));
        }
        drop(code);
        if let Err(code) = parent.storage_grant(server, 0, 0, 0)? {
            self.stop(0, server, world, parent)?;
            return Err(Error(code as i64));
        }
        self.saved_storage[0] = Some((0, 0));
        // Distribution resource ceilings: a declared app may host a nested desktop.
        // This conveys no executable, storage or device authority by itself.
        self.desktop = true;
        for slot in 1..BACKGROUND {
            #[cfg(feature = "input-producer-probe")]
            {
                self.allowed_imports[slot] = Some(
                    cathedral_platform_contracts::imports::ALL
                        | cathedral_platform_contracts::imports::INPUT_PRODUCER,
                );
            }
            self.standing[slot] = Some(Standing {
                pages: [saved.app_pages, saved.app_readers],
                channels: 40,
                descendants: 5,
                cpu: 0,
                surfaces: saved.app_surfaces,
                place: Some((rect(rects[1]), saved.keys)),
            });
        }
        let placement = world
            .surface_range(
                0,
                world
                    .backed_surfaces()
                    .saturating_sub(config::LAUNCHER_SURFACES),
                config::LAUNCHER_SURFACES,
            )
            .and_then(|()| world.place(0, rect(rects[0]), saved.keys));
        if let Err(code) = placement {
            self.desktop = false;
            self.standing = [None; CHILDREN];
            self.stop(0, server, world, parent)?;
            surface_limit(world, 1, 0).map_err(|c| Error(c as i64))?;
            return Err(Error(code as i64));
        }
        if let Err(error) = self.background(background, parent, world, server) {
            time::storage_sample("background-start-refused", &[error.0 as u64]);
            self.startup_recovery.policy.background_failed();
        }
        super::super::requests::flush(parent, world)?;
        self.notify(world)?;
        parent.grant_recovery(0, server.incarnation(0));
        world.grant_layout(0, server.incarnation(0).expect("configured launcher"));
        self.management
            .grant(0, server.incarnation(0).unwrap(), 1)
            .map_err(|c| Error(c as i64))?;
        self.launcher = Some(recovery::Launcher::new(
            plan,
            self.slots[0].as_ref().unwrap().task.raw(),
            server.incarnation(0).unwrap(),
        ));
        cathedral_user_runtime::write(b"Cathedral matrix: configured from realm paths\n")?;
        Ok(())
    }
    /// Background is a separately installed role at a realm-local path. Replacing
    /// its executable replaces the background; it receives interaction only.
    /// `known` is the entry the configure listing already named; a recovery
    /// relaunch passes None and looks the path up afresh.
    pub(super) fn background(
        &mut self,
        known: Option<u64>,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let (object, mut code) = parent.local_storage(|scope| {
            scope.bind().map_err(|c| Error(c as i64))?;
            let object = match known {
                Some(object) => object,
                None => scope.lookup(b"Background").map_err(|c| Error(c as i64))?,
            };
            Ok((object, read(scope, object, None)?))
        })?;
        let plan = parent::LaunchRequest {
            slot: BACKGROUND,
            object,
            scope: 0,
            argument: cathedral_desktop_matrix_contracts::matrix::launch_argument(0, object),
            channels: 0,
            descendants: 0,
            cpu: 0,
            pages: Some([4, 0]),
            stamp: 0,
            imports: cathedral_platform_contracts::imports::INTERACTION,
        };
        self.run_object(plan, &mut code, parent, server, world)?;
        let result = world.surface_range(BACKGROUND, 0, 1).and_then(|()| {
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
        });
        if let Err(code) = result {
            self.stop(BACKGROUND, server, world, parent)?;
            return Err(Error(code as i64));
        }
        cathedral_user_runtime::write(
            b"Cathedral matrix: background admitted with presentation only\n",
        )?;
        Ok(())
    }
    pub(in crate::service) fn apply_layout(
        &mut self,
        request: &Request,
        words: wire::Packet,
        parent: &mut Parent,
        world: &mut World,
    ) -> Result<wire::Packet, Error> {
        if !world.layout_allowed(request.client.unwrap(), request.incarnation, words[1]) {
            return Ok(wire::error(abi::DENIED));
        }
        if self.loss.is_some() || self.pending.iter().any(Option::is_some) {
            return Ok(wire::error(abi::BUSY));
        }
        let [wire::APPLY_LAYOUT, epoch, margin, bar, gap, 0, 0, 0] = words else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        let grant = parent.grant;
        let Some(rects) = policy::rectangles(margin, bar, gap, grant.width, grant.height) else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        let pair = rects.map(rect);
        let rects = core::array::from_fn(|i| {
            if i == 0 {
                pair[0]
            } else if i == BACKGROUND {
                Rect {
                    x: 0,
                    y: 0,
                    width: grant.width,
                    height: grant.height,
                }
            } else {
                pair[1]
            }
        });
        let rects = match world.layout(request.client.unwrap(), request.incarnation, epoch, rects) {
            Ok(rects) => rects,
            Err(code) => return Ok(wire::error(code)),
        };
        for (slot, next) in rects.iter().enumerate().take(BACKGROUND).skip(1) {
            if let Some(standing) = &mut self.standing[slot]
                && let Some((rect, _)) = &mut standing.place
            {
                *rect = *next;
            }
        }
        super::super::requests::flush(parent, world)?;
        for slot in 0..CHILDREN {
            super::super::requests::replay(parent, world, slot)?;
        }
        Ok([0; 8])
    }
    pub(super) fn restore_storage(
        &mut self,
        parent: &mut Parent,
        server: &Server,
    ) -> Result<(), Error> {
        for slot in 0..CHILDREN {
            if self.slots[slot].is_some()
                && let Some((source, scope)) = self.saved_storage[slot]
            {
                match parent.storage_grant(server, slot, source, scope)? {
                    Ok(()) => {}
                    // storage_grant refreshes upstream state before checking.
                    // A matching live grant needs no new binding incarnation.
                    Err(code)
                        if code == abi::BUSY
                            && parent.storage_binding(server, slot) == Some((source, scope)) => {}
                    Err(code) => return Err(Error(code as i64)),
                }
            }
        }
        Ok(())
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
/// Read a realm object whole in one request. `expected` is the exact length
/// a fixed record must have; a longer or shorter object is refused.
pub(super) fn read(
    scope: &mut Scope,
    id: u64,
    expected: Option<usize>,
) -> Result<cathedral_user_runtime::memory::Object, Error> {
    let len = expected.map_or(0, |n| n as u64);
    let (reply, object) = scope
        .request(storage::Request::new(storage::BULK, id, 0, 0, len), None)
        .map_err(|c| Error(c as i64))?;
    let reply = storage::Reply::decode(&reply).map_err(|c| Error(c as i64))?;
    if expected.is_some_and(|n| reply.length != n as u64) {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    object.ok_or(Error(abi::IO_ERROR as i64))
}
