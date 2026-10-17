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
            server,
            world,
        )
    }
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
        let requested_frames = self.next_frames[slot].take();
        let [private_pages, reader_pages] = self.next_pages[slot].take().unwrap_or([0, 0]);
        if images >> abi::MAX_LAUNCHES != 0 {
            return Err(Error(abi::DENIED as i64));
        }
        let default_frames = descendants
            .checked_add(1)
            .and_then(|n| n.checked_mul(256))
            .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let target = Executable::at(image)?.reserve_funded(
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
            self.install(slot, target, &[], argument, true, server, world)
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
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        let port = target.port()?;
        if self.separate_storage && !server.available(slot + 3) {
            return Err(Error(abi::BUSY as i64));
        }
        let connections = connections::Connections::create(self.separate_storage)?;
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
            reservation: dynamic.then_some(target),
            control: Some(control),
            stale: 0,
            data: connections.data,
            storage,
            notify: connections.notify,
        });
        Ok(())
    }
}
