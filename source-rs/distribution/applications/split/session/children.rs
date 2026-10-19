//! Load declared code from our realm, reserve bounded tasks, withhold persistence.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::{
    matrix::admission::{self, Admitted},
    resources::Limits,
    storage::Scope,
    task::{Executable, Launch},
};
pub struct Children {
    pub slots: [Option<Admitted>; 2],
    launches: [Option<Launch>; 2],
    pub uncertain: [bool; 2],
}
impl Children {
    pub fn empty() -> Self {
        Self {
            slots: [None, None],
            launches: [None, None],
            uncertain: [false; 2],
        }
    }
    pub fn start(&mut self, storage: Pair, server: &mut Server) -> Result<(), Error> {
        let mut scope = Scope::new(storage);
        scope.bind().map_err(|c| Error(c as i64))?;
        let id = scope.lookup(b"Notes").map_err(|c| Error(c as i64))?;
        let head = scope
            .call(storage::Request::new(storage::HEAD, id, 0, 0, 0))
            .map_err(|c| Error(c as i64))?;
        let (reply, object) = scope
            .request(
                storage::Request::new(storage::BULK, id, head.revision, 0, head.length),
                None,
            )
            .map_err(|c| Error(c as i64))?;
        storage::Reply::decode(&reply).map_err(|c| Error(c as i64))?;
        let mut code = object.ok_or(Error(abi::IO_ERROR as i64))?;
        let declaration = imports::from_elf(code.map()?)
            .map_err(|_| Error(abi::BAD_EXECUTABLE as i64))?
            .ok_or(Error(abi::BAD_EXECUTABLE as i64))?;
        code.unmap()?;
        let granted = imports::bind(declaration.entries(), imports::INTERACTION)
            .map_err(|_| Error(imports::MISSING as i64))?;
        if !declaration.brokered || granted != imports::INTERACTION {
            return Err(Error(abi::DENIED as i64));
        }
        for slot in 0..2 {
            let launch = Executable::from_object(&code, 0)?.reserve(
                Limits {
                    private_pages: 32,
                    reader_pages: 4,
                    channels: 0,
                },
                0,
                abi::MAX_ADMISSION_FRAMES as u64,
            )?;
            let child = match admission::spawn_brokered(launch, 0, &[], granted) {
                Ok(child) => child,
                Err(e) => {
                    launch.release()?;
                    return Err(e);
                }
            };
            server.install(slot, child.data)?;
            self.launches[slot] = Some(launch);
            self.slots[slot] = Some(child);
        }
        cathedral_user_runtime::write(
            b"Cathedral Split: two Notes admitted; storage withheld from children\n",
        )?;
        Ok(())
    }
    pub fn notify(&self, world: &mut World) -> Result<(), Error> {
        for slot in 0..2 {
            if let Some(child) = &self.slots[slot]
                && child
                    .services
                    .as_ref()
                    .is_some_and(|s| s.interaction_ready())
                && world.take_notification(slot)
            {
                match child.notify.send(&wire::encode([0; 8])) {
                    Ok(()) => (),
                    Err(Error(c))
                        if c == abi::WOULD_BLOCK as i64 || c == abi::PEER_CLOSED as i64 => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }
    pub fn stop(&mut self, slot: usize, server: &mut Server) -> Result<(), Error> {
        if let Some(child) = self.slots[slot].take() {
            server.remove(slot);
            child.task.cancel()?;
            child.task.wait()?;
            child.close_connections()?;
            self.launches[slot].take().unwrap().release()?;
        }
        Ok(())
    }
}
