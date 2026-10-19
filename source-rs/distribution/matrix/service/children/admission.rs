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
            None,
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
            None,
            server,
            world,
        )
    }
    /// Code from a sealed object read out of this matrix's realm. It lends only
    /// explicitly owner-allowed executable grants (none by default), and it
    /// receives only the owner-selected startup argument and environment.
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
        self.requested_surfaces[plan.slot] = None;
        let imports::Bound {
            binding,
            profile,
            brokered,
            resources,
        } = imports::resolve(object, plan.imports)?;
        if brokered && !self.separate_storage {
            return Err(Error(abi::DENIED as i64));
        }
        let storage = binding & cathedral_platform_contracts::imports::STORAGE != 0;
        let executable = Executable::from_object(object, 0)?;
        let (mut pages, frames) = match plan.pages {
            Some(pages) => (pages, None),
            None => (
                self.next_pages[plan.slot].take().unwrap_or([0, 0]),
                self.next_frames[plan.slot].take(),
            ),
        };
        if let Some([private, readers, surfaces]) = profile {
            if private > pages[0]
                || readers > pages[1]
                || resources.is_some_and(|(c, d)| c > plan.channels || d > plan.descendants)
            {
                return Err(Error(abi::NO_MEMORY as i64));
            }
            pages = [private, readers];
            self.requested_surfaces[plan.slot] = Some(surfaces);
        } else if self.desktop && (1..BACKGROUND).contains(&plan.slot) {
            pages = [pages[0].min(128), pages[1].min(128)];
            self.requested_surfaces[plan.slot] = Some(10);
        }
        // Reserve geometry before spawning so a nested matrix cannot start with
        // less presentation capacity than it requested.
        if self.desktop
            && let Some(count) = self.requested_surfaces[plan.slot]
        {
            let ceiling = self.standing[plan.slot].map_or(0, |s| s.surfaces);
            if count > ceiling {
                return Err(Error(abi::NO_MEMORY as i64));
            }
            self.place_surface_range(world, plan.slot, count as usize)
                .map_err(|c| Error(c as i64))?;
        }
        let (channels, descendants) = resources
            .map_or((plan.channels, plan.descendants), |(c, d)| {
                (c.min(plan.channels), d.min(plan.descendants))
            });
        let admitted = self.admit(
            plan.slot,
            executable,
            pages,
            frames,
            [
                plan.argument,
                channels,
                descendants,
                self.allowed_images[plan.slot],
                plan.cpu,
            ],
            storage && self.separate_storage,
            brokered.then_some(binding),
            server,
            world,
        );
        if admitted.is_err() && self.desktop {
            world
                .surface_range(plan.slot, 0, 0)
                .map_err(|c| Error(c as i64))?;
        }
        admitted?;
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
        services: Option<u64>,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        let default_frames = descendants
            .checked_add(1)
            .and_then(|n| n.checked_mul(256))
            .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let target = executable
            .reserve_funded(
                Limits {
                    private_pages,
                    reader_pages,
                    channels,
                },
                descendants,
                256,
                requested_frames.unwrap_or(default_frames),
            )
            .inspect_err(|error| {
                refused(
                    "matrix-reserve-refused",
                    slot,
                    error,
                    [private_pages, reader_pages, channels],
                );
            })?;
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
                services,
                server,
                world,
            )
        })();
        if let Err(error) = &result {
            refused(
                "matrix-start-refused",
                slot,
                error,
                [private_pages, reader_pages, channels],
            );
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
        services: Option<u64>,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        if separate_storage && !server.available(slot + STORAGE_BASE) {
            return Err(Error(abi::BUSY as i64));
        }
        use cathedral_user_runtime::matrix::admission;
        let admitted = match services {
            Some(granted) => admission::spawn_brokered(target, argument, loans, granted)?,
            None => admission::spawn(target, argument, loans, separate_storage)?,
        };
        world.disconnected(slot);
        server
            .install(slot, admitted.data)
            .expect("validated local client slot changed");
        let storage = admitted.storage;
        if let Some(pair) = storage {
            server
                .install(slot + STORAGE_BASE, pair)
                .expect("child catalog slot changed");
        }
        self.slots[slot] = Some(Owned {
            services: admitted.services,
            task: admitted.task,
            outcome: None,
            disconnected: false,
            reservation: dynamic.then_some(target),
            control: Some(admitted.control),
            stale: 0,
            data: admitted.data,
            storage,
            notify: admitted.notify,
        });
        if slot == 0 {
            // A separately admitted task supersedes any old configured launcher's
            // queued recovery. CONFIGURE arms its policy only after full success.
            self.launcher = None;
        }
        Ok(())
    }
}

/// Local budget diagnostics only; no other process's resources or names leak.
fn refused(stage: &str, slot: usize, error: &Error, requested: [u64; 3]) {
    time::storage_sample(
        stage,
        &[
            slot as u64,
            error.0 as u64,
            requested[0],
            requested[1],
            requested[2],
        ],
    );
    if let Ok(budget) = cathedral_user_runtime::resources::budget() {
        time::storage_sample(
            "matrix-local-capacity",
            &[
                budget.limits.private_pages,
                budget.used.private_pages,
                budget.limits.reader_pages,
                budget.used.reader_pages,
                budget.limits.channels,
                budget.used.channels,
            ],
        );
    }
}
