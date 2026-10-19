//! This program owns three launch loans; shared admission supplies connection custody.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::World;
use cathedral_user_runtime::{
    Error,
    matrix::admission::{self, Admitted},
    resources,
    server::Server,
    task::Launch,
};

pub struct Children(pub [Option<Admitted>; 3]);
impl Children {
    pub fn start(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World<3>,
    ) -> Result<(), Error> {
        if self.0[slot].is_some() || !server.available(slot) {
            return Err(Error(abi::BUSY as i64));
        }
        let child = admission::spawn(Launch::at(slot as u64)?, 0, &[], false)?;
        world.disconnected(slot);
        server
            .install(slot, child.data)
            .expect("validated empty slot");
        self.0[slot] = Some(child);
        Ok(())
    }
    pub fn stop(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World<3>,
    ) -> Result<(), Error> {
        let child = self.0[slot].take().ok_or(Error(abi::PEER_CLOSED as i64))?;
        server.remove(slot);
        world.disconnected(slot);
        child.task.cancel()?;
        child.task.wait()?;
        child.close_connections()
    }
    /// Repeated failures must return both endpoint slots and channel charges.
    pub fn prove_rollback(&self, expected: u64) -> Result<(), Error> {
        let before = resources::budget()?;
        if expected == abi::NO_MEMORY {
            assert_eq!(before.limits.channels - before.used.channels, 2);
        }
        for _ in 0..16 {
            assert!(
                matches!(admission::spawn(Launch::at(0)?, 0, &[], false), Err(Error(code)) if code == expected as i64)
            );
            assert_eq!(resources::budget()?, before);
        }
        Ok(())
    }
}
