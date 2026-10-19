//! The owner selects code. This provider supplies only the retired launcher's
//! local view and a connection-bound, single-use request for another attempt.
use super::*;
impl Children {
    pub(in crate::service::children) fn set_fallback(
        &mut self,
        image: u64,
        argument: u64,
    ) -> Result<(), Error> {
        self.fallback = if image == 0 {
            None
        } else {
            Some(Fallback {
                image: cathedral_user_runtime::task::Executable::at(image - 1)?,
                argument,
            })
        };
        // Revocation cannot be undone by a queued fallback request.
        self.launcher = None;
        Ok(())
    }
    pub(in crate::service) fn retry_launcher(&mut self, request: &Request) -> wire::Packet {
        let Some(launcher) = &mut self.launcher else {
            return wire::error(abi::DENIED);
        };
        if self.fallback.is_none()
            || request.client != Some(0)
            || request.incarnation != launcher.incarnation
            || launcher.phase != Phase::Fallback
            || self.slots[0].as_ref().map(|child| child.task.raw()) != Some(launcher.task)
        {
            return wire::error(abi::DENIED);
        }
        launcher.phase = Phase::Requested;
        [0; 8]
    }
    pub(super) fn show_fallback(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<bool, Error> {
        let mut launcher = self.launcher.take().unwrap();
        let Some(fallback) = self.fallback else {
            return Ok(false);
        };
        let (placement, _) = launcher.snapshot.unwrap();
        let result = self
            .admit(
                0,
                fallback.image,
                launcher.plan.pages.unwrap(),
                None,
                [fallback.argument, 0, 0, 0, 0],
                false,
                server,
                world,
            )
            .and_then(|()| {
                world
                    .restore(placement, server.incarnation(0).unwrap(), false)
                    .map_err(|c| Error(c as i64))
            });
        if let Err(error) = result {
            if self.slots[0].is_some() {
                self.stop(0, server, world, parent)?;
            }
            time::storage_sample("launcher-fallback-refused", &[error.0 as u64]);
            return Ok(true);
        }
        launcher.task = self.slots[0].as_ref().unwrap().task.raw();
        launcher.incarnation = server.incarnation(0).unwrap();
        launcher.phase = Phase::Fallback;
        self.launcher = Some(launcher);
        self.notify(world)?;
        cathedral_user_runtime::write(b"Cathedral matrix: launcher fallback admitted\n")?;
        Ok(true)
    }
}
