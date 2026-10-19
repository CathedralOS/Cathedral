//! Apply this matrix's saved arrangement to empty slots. The enclosing owner
//! triggers startup; it never chooses our internal launcher, placement or budgets.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::storage::Scope;

impl Children {
    pub(super) fn configure(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.slots.iter().any(Option::is_some) || parent.storing() {
            return Err(Error(abi::BUSY as i64));
        }
        #[cfg(feature = "startup-fault")]
        if self.slots.iter().all(Option::is_none) {
            panic!("deliberate matrix startup failure");
        }
        parent.refresh(world)?;
        let (saved, launcher, mut code) = parent.local_storage(|scope| {
            scope.bind().map_err(|c| Error(c as i64))?;
            let object = scope.lookup(config::NAME).map_err(|c| Error(c as i64))?;
            let mut bytes = read(scope, object, Some(config::SIZE))?;
            let saved = config::Config::decode(&bytes.map()?[..config::SIZE])
                .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
            drop(bytes);
            let launcher = scope
                .lookup(storage::metadata::text(&saved.launcher))
                .map_err(|c| Error(c as i64))?;
            let code = read(scope, launcher, None)?;
            Ok((saved, launcher, code))
        })?;
        if saved.keys & !parent.grant.keys != 0 {
            return Err(Error(abi::DENIED as i64));
        }
        let rects = saved
            .rectangles(parent.grant.width, parent.grant.height)
            .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let plan = parent::LaunchRequest {
            slot: 0,
            object: launcher,
            scope: 0,
            argument: 0,
            channels: 0,
            descendants: 0,
            cpu: 0,
            pages: Some([saved.launcher_pages, 0]),
            stamp: 0,
            imports: cathedral_platform_contracts::imports::ALL,
        };
        if !self.run_object(plan, &mut code, server, world)? {
            self.stop(0, server, world, parent)?;
            return Err(Error(abi::DENIED as i64));
        }
        drop(code);
        if let Err(code) = parent.storage_grant(server, 0, 0, 0)? {
            self.stop(0, server, world, parent)?;
            return Err(Error(code as i64));
        }
        self.saved_storage[0] = Some((0, 0));
        // Distribution resource policy: any app may host two bounded children.
        // This conveys no executable, storage or device authority by itself.
        self.standing[1] = Some(Standing {
            pages: [saved.app_pages, saved.app_readers],
            channels: 10,
            descendants: 2,
            cpu: 0,
            surfaces: saved.app_surfaces,
            place: Some((rect(rects[1]), saved.keys)),
        });
        let placement = surface_limit(world, 0, 5)
            .and_then(|()| surface_limit(world, 1, saved.app_surfaces as usize))
            .and_then(|()| world.place(0, rect(rects[0]), saved.keys));
        if let Err(code) = placement {
            self.standing[1] = None;
            self.stop(0, server, world, parent)?;
            surface_limit(world, 1, 0).map_err(|c| Error(c as i64))?;
            return Err(Error(code as i64));
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
        let [wire::APPLY_LAYOUT, epoch, margin, strip, gap, 0, 0, 0] = words else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        let grant = parent.grant;
        let Some(rects) = config::rectangles(margin, strip, gap, grant.width, grant.height) else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        let rects = rects.map(rect);
        let rects = match world.layout(request.client.unwrap(), request.incarnation, epoch, rects) {
            Ok(rects) => rects,
            Err(code) => return Ok(wire::error(code)),
        };
        if let Some(standing) = &mut self.standing[1]
            && let Some((rect, _)) = &mut standing.place
        {
            *rect = rects[1];
        }
        super::super::requests::flush(parent, world)?;
        for slot in 0..2 {
            super::super::requests::replay(parent, world, slot)?;
        }
        Ok([0; 8])
    }
    pub(super) fn restore_storage(
        &mut self,
        parent: &mut Parent,
        server: &Server,
    ) -> Result<(), Error> {
        for slot in 0..2 {
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
fn read(
    scope: &mut Scope,
    id: u64,
    expected: Option<usize>,
) -> Result<cathedral_user_runtime::memory::Object, Error> {
    let head = scope
        .call(storage::Request::new(storage::HEAD, id, 0, 0, 0))
        .map_err(|c| Error(c as i64))?;
    if head.length == 0
        || head.length > storage::MAX_BYTES as u64
        || expected.is_some_and(|n| head.length != n as u64)
    {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    let (reply, object) = scope
        .request(
            storage::Request::new(storage::BULK, id, head.revision, 0, head.length),
            None,
        )
        .map_err(|c| Error(c as i64))?;
    storage::Reply::decode(&reply).map_err(|c| Error(c as i64))?;
    object.ok_or(Error(abi::IO_ERROR as i64))
}
