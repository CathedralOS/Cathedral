// A hosted matrix learns what it is from its realm. A `Matrix` configuration
// makes it a workspace that runs its own launcher; a `Login` picker beside a
// `Users` folder makes it a desktop that offers those users; a realm with
// neither, or no realm at all, leaves it to its owner's commands. No launch
// argument or build selects this. The nested desktop starts a scoped picker,
// then retires it for the chosen user matrix, which runs the same CONFIGURE
// path as the top-level desktop.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config;
/// What the realm asks this matrix to become.
enum Shape {
    Workspace { config: u64 },
    Picker { login: u64, users: u64 },
    Hosted,
}
impl Children {
    pub(in crate::service) fn startup(
        &mut self,
        mode: &mut Startup,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if *mode == Startup::Realm {
            parent.refresh(world)?;
            if parent.grant.width == 0 {
                return Ok(());
            }
            match self.realm_shape(parent)? {
                Shape::Workspace { config } => {
                    self.configure_object(Some(config), parent, world, server)?;
                }
                Shape::Picker { login, users } => {
                    self.start_picker(parent, world, server, login, users)?;
                }
                Shape::Hosted => {}
            }
            *mode = Startup::Hosted;
        }
        self.recover_startup(parent, world, server)?;
        // A selected user's admission and picker retirement are sequenced in
        // children/launch/launch.rs, before that user's first placement is published.
        Ok(())
    }
    /// Read the realm's shape through this matrix's own binding, from one
    /// listing of its entries. No binding, or a realm with none of the stock
    /// names, means an owner-driven matrix.
    fn realm_shape(&self, parent: &mut Parent) -> Result<Shape, Error> {
        let inspected = parent.local_storage(|scope| {
            if scope.bind().is_err() {
                return Ok(Shape::Hosted);
            }
            let mut ids = [None; 3];
            scope
                .named(&[config::NAME, b"Login", b"Users"], &mut ids)
                .map_err(|c| Error(c as i64))?;
            Ok(match ids {
                [Some(config), _, _] => Shape::Workspace { config },
                [None, Some(login), Some(users)] => Shape::Picker { login, users },
                _ => Shape::Hosted,
            })
        });
        match inspected {
            Ok(shape) => Ok(shape),
            // Headless fixtures and the root adapter have no upstream binding
            // to ask; their owners configure them explicitly.
            Err(Error(code)) if code == abi::DENIED as i64 || code == abi::PEER_CLOSED as i64 => {
                Ok(Shape::Hosted)
            }
            Err(error) => Err(error),
        }
    }
    pub(super) fn start_picker(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
        picker: u64,
        users: u64,
    ) -> Result<(), Error> {
        let picker_mode = self.startup_recovery.policy.picker_started();
        self.startup_recovery.picker = Some((picker, users));
        let mut code = parent.local_storage(|scope| {
            scope.bind().map_err(|c| Error(c as i64))?;
            configuration::read(scope, picker, None)
        })?;
        let capacity = world.backed_surfaces();
        if capacity < wire::config::HOST_SURFACES {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        self.run_object(
            parent::LaunchRequest {
                slot: 0,
                object: picker,
                scope: users,
                argument: cathedral_desktop_matrix_contracts::matrix::launch_argument(
                    picker_mode,
                    picker,
                ),
                channels: 2,
                descendants: 0,
                cpu: 0,
                pages: Some([
                    48,
                    cathedral_desktop_matrix_contracts::matrix::CHOOSER_READERS,
                ]),
                stamp: 0,
                imports: cathedral_platform_contracts::imports::ALL,
            },
            &mut code,
            parent,
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
