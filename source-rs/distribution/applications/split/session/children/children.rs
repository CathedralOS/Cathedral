// Start ordinary leaves or descend into another matrix; retain ownership of
// their channels and reservations until collection. Image validation lives in
// image.rs, while nesting.rs owns the persistent copy and child-matrix policy.
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
    pub fn start(
        &mut self,
        storage: Pair,
        server: &mut Server,
    ) -> Result<super::storage::Storage, Error> {
        let mut scope = Scope::new(storage);
        scope.bind().map_err(|c| Error(c as i64))?;
        let roots = super::storage::open(&mut scope)?;
        let mut forwarding = super::storage::Storage::new(storage, scope.realm(), roots);
        let id = scope.lookup(b"Notes").map_err(|c| Error(c as i64))?;
        let (code, granted) = image::read(&mut scope, id)?;
        for slot in 0..2 {
            let launch = Executable::from_object(&code, 0)?.reserve(
                Limits {
                    // Omega host (9) + old/new text (24) + small retained
                    // decorations and storage publication, all app-charged.
                    private_pages: 40,
                    reader_pages: 4,
                    channels: 0,
                },
                0,
                abi::MAX_ADMISSION_FRAMES as u64,
            )?;
            let child = match admission::spawn_brokered(launch, 0, &[], granted) {
                Ok(child) => child,
                Err(e) => {
                    let budget = cathedral_user_runtime::ipc::budget().unwrap();
                    cathedral_user_runtime::time::storage_sample(
                        "split-admission-refused",
                        &[
                            slot as u64,
                            e.0 as u64,
                            budget.channel_limit,
                            budget.charged_channels,
                            budget.handle_limit,
                            budget.charged_handles,
                        ],
                    );
                    launch.release()?;
                    return Err(e);
                }
            };
            server.install(slot, child.data)?;
            server.install(slot + 2, child.storage.unwrap())?;
            forwarding.install(slot, server.incarnation(slot + 2).unwrap());
            self.launches[slot] = Some(launch);
            self.slots[slot] = Some(child);
        }
        cathedral_user_runtime::write(
            b"Cathedral Split: two Notes admitted with separate persistent realms\n",
        )?;
        Ok(forwarding)
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
            server.remove(slot + 2);
            child.task.cancel()?;
            child.task.wait()?;
            child.close_connections()?;
            self.launches[slot].take().unwrap().release()?;
        }
        Ok(())
    }
}
