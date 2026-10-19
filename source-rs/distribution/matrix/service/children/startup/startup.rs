// The nested desktop starts a scoped picker, then retires it for the chosen
// user matrix. That user runs the same CONFIGURE path as the top-level desktop.
use super::*;
impl Children {
    pub(in crate::service) fn startup(
        &mut self,
        mode: &mut Startup,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if *mode != Startup::Hosted {
            parent.refresh(world)?;
            if parent.grant.width == 0 {
                return Ok(());
            }
            if *mode == Startup::Workspace {
                self.configure(parent, world, server)?;
            } else {
                self.start_picker(parent, world, server)?;
            }
            *mode = Startup::Hosted;
        }
        if self.login && !self.entered && self.slots[1].is_some() {
            self.stop(0, server, world, parent)?;
            // Expansion keeps the child's rank base unchanged: the initial
            // allowance starts at zero; the picker was stacked above it.
            world
                .surface_range(1, 0, world.backed_surfaces())
                .map_err(|c| Error(c as i64))?;
            self.entered = true;
            self.notify(world)?;
            cathedral_user_runtime::write(
                b"Cathedral nested desktop: picker retired; user owns viewport\n",
            )?;
        }
        Ok(())
    }
    fn start_picker(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let parent_keys = parent.grant.keys;
        let (picker, users, mut code) = parent.local_storage(|scope| {
            scope.bind().map_err(|c| Error(c as i64))?;
            let (picker, users) =
                realm::prepare(scope, parent_keys).map_err(|c| Error(c as i64))?;
            let code = configuration::read(scope, picker, None)?;
            Ok((picker, users, code))
        })?;
        let capacity = world.backed_surfaces();
        if capacity < 24 {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        self.run_object(
            parent::LaunchRequest {
                slot: 0,
                object: picker,
                scope: users,
                argument: 1,
                channels: 0,
                descendants: 0,
                cpu: 0,
                pages: Some([48, 0]),
                stamp: 0,
                imports: cathedral_platform_contracts::imports::ALL,
            },
            &mut code,
            server,
            world,
        )?;
        parent
            .storage_grant(server, 0, 0, users)?
            .map_err(|c| Error(c as i64))?;
        self.saved_storage[0] = Some((0, users));
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
        self.notify(world)?;
        cathedral_user_runtime::write(b"Cathedral nested desktop: scoped picker ready\n")?;
        Ok(())
    }
}
