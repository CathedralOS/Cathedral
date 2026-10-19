//! Select local launch grants, create the child's parent connection, admit, install.
use super::*;
use cathedral_user_runtime::{
    resources::Limits,
    task::{Executable, Launch},
};
impl Children {
    pub(super) fn start(
        &mut self,
        slot: usize,
        launch: u64,
        mask: u64,
        argument: u64,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        if self.slots[slot].is_some() || !server.available(slot) {
            return Err(Error(abi::BUSY as i64));
        }
        if mask >> abi::MAX_LAUNCHES != 0 {
            return Err(Error(abi::DENIED as i64));
        }
        let target = Launch::at(launch)?;

        let mut loans = [Launch::from_raw(0); abi::MAX_LAUNCHES];
        let mut count = 0;
        for index in 0..abi::MAX_LAUNCHES {
            if mask & (1 << index) != 0 {
                loans[count] = Launch::at(index as u64)?;
                count += 1;
            }
        }
        self.install(
            slot,
            target,
            &loans[..count],
            argument,
            false,
            self.separate_storage,
            server,
            world,
        )
    }
    /// Code from the owner's catalogue, with the catalogue grants `images` lends.
    pub(super) fn run(
        &mut self,
        slot: usize,
        [image, argument, channels, descendants, images, cpu]: [u64; 6],
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        if self.slots[slot].is_some() || !server.available(slot) {
            return Err(Error(abi::BUSY as i64));
        }
        if images >> abi::MAX_LAUNCHES != 0 {
            return Err(Error(abi::DENIED as i64));
        }
        let executable = Executable::at(image)?;
        let pages = self.next_pages[slot].take().unwrap_or([0, 0]);
        let frames = self.next_frames[slot].take();
        self.admit(
            slot,
            executable,
            pages,
            frames,
            [argument, channels, descendants, images, cpu],
            self.separate_storage,
            server,
            world,
        )
    }
    /// Code from a sealed object read out of this matrix's realm. It lends only
    /// explicitly owner-allowed executable grants (none by default), and it
    /// enters with no first argument: a program's shape comes from what it is
    /// handed, not from a role word.
    pub(super) fn run_object(
        &mut self,
        plan: super::parent::LaunchRequest,
        object: &mut cathedral_user_runtime::memory::Object,
        server: &mut Server,
        world: &mut World,
    ) -> Result<bool, Error> {
        if self.slots[plan.slot].is_some() || !server.available(plan.slot) {
            return Err(Error(abi::BUSY as i64));
        }
        let binding = imports::resolve(object, plan.imports)?;
        let storage = binding & cathedral_platform_contracts::imports::STORAGE != 0;
        let executable = Executable::from_object(object, 0)?;
        let (pages, frames) = match plan.pages {
            Some(pages) => (pages, None),
            None => (
                self.next_pages[plan.slot].take().unwrap_or([0, 0]),
                self.next_frames[plan.slot].take(),
            ),
        };
        self.admit(
            plan.slot,
            executable,
            pages,
            frames,
            [
                plan.argument,
                plan.channels,
                plan.descendants,
                self.allowed_images[plan.slot],
                plan.cpu,
            ],
            storage && self.separate_storage,
            server,
            world,
        )?;
        Ok(storage)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn admit(
        &mut self,
        slot: usize,
        executable: Executable,
        [private_pages, reader_pages]: [u64; 2],
        requested_frames: Option<u64>,
        [argument, channels, descendants, images, cpu]: [u64; 5],
        separate_storage: bool,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        let default_frames = descendants
            .checked_add(1)
            .and_then(|n| n.checked_mul(256))
            .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let target = executable.reserve_funded(
            Limits {
                private_pages,
                reader_pages,
                channels,
            },
            descendants,
            256,
            requested_frames.unwrap_or(default_frames),
        )?;
        let result = (|| {
            target.assign_cpu(cpu)?;
            for index in 0..abi::MAX_LAUNCHES {
                if images & (1 << index) != 0 {
                    target.allow_executable(Executable::at(index as u64)?)?;
                }
            }
            self.install(
                slot,
                target,
                &[],
                argument,
                true,
                separate_storage,
                server,
                world,
            )
        })();
        if result.is_err() {
            target.release()?;
        }
        result
    }
    #[allow(clippy::too_many_arguments)]
    fn install(
        &mut self,
        slot: usize,
        target: Launch,
        loans: &[Launch],
        argument: u64,
        dynamic: bool,
        separate_storage: bool,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        let port = target.port()?;
        if separate_storage && !server.available(slot + 3) {
            return Err(Error(abi::BUSY as i64));
        }
        let connections = connections::Connections::create(separate_storage)?;
        let child = match target.spawn_tree(argument, connections.initial(), loans) {
            Ok(child) => child,
            Err(error) => {
                connections.rollback()?;
                return Err(error);
            }
        };
        let control = match port.connect() {
            Ok(pair) => pair,
            Err(error) => {
                child.cancel()?;
                child.wait()?;
                connections.close_parent()?;
                return Err(error);
            }
        };
        world.disconnected(slot);
        server
            .install(slot, connections.data)
            .expect("validated local client slot changed");
        let storage = connections.storage;
        if let Some(pair) = storage {
            server
                .install(slot + 3, pair)
                .expect("child catalog slot changed");
        }
        self.slots[slot] = Some(Owned {
            task: child,
            outcome: None,
            disconnected: false,
            reservation: dynamic.then_some(target),
            control: Some(control),
            stale: 0,
            data: connections.data,
            storage,
            notify: connections.notify,
        });
        if slot == 0 {
            // A separately admitted task supersedes any old configured launcher's
            // queued recovery. CONFIGURE arms its policy only after full success.
            self.launcher = None;
        }
        Ok(())
    }
}
