//! Separate control-channel failure from task death, then reclaim a child and
//! its delegated funding. Receipts follow the kernel outcome of that exact task.
use super::*;

impl Children {
    pub(in crate::service) fn disconnected(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<(), Error> {
        self.capture_recovery(slot, world, parent);
        parent.storage_close(slot)?;
        if self.pending[slot].is_some() {
            if let Some(child) = &mut self.slots[slot] {
                child.disconnected = true;
            }
            parent.batches[slot] = Default::default();
            world.disconnected(slot);
            return Ok(());
        }
        if self.slots[slot].is_some() {
            self.stop(slot, server, world, parent)?;
        } else {
            parent.batches[slot] = Default::default();
            world.disconnected(slot);
        }
        Ok(())
    }
    /// Observe the kernel's exact task outcome without cancelling a live child.
    /// A successful wait consumes it, so retain it until connection/funding cleanup.
    pub(in crate::service) fn completed(&mut self, slot: usize) -> Result<bool, Error> {
        let Some(child) = &mut self.slots[slot] else {
            return Ok(true);
        };
        if child.outcome.is_none() {
            match child.task.wait_until(time::now()?) {
                Ok(outcome) => child.outcome = Some(outcome),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => return Ok(false),
                Err(error) => return Err(error),
            }
        }
        Ok(true)
    }
    pub(in crate::service) fn stop(
        &mut self,
        slot: usize,
        server: &mut Server,
        world: &mut World,
        parent: &mut Parent,
    ) -> Result<(), Error> {
        self.capture_recovery(slot, world, parent);
        parent.storage_close(slot)?;
        let child = self.slots[slot]
            .take()
            .ok_or(Error(abi::PEER_CLOSED as i64))?;
        self.saved_storage[slot] = None;
        self.observations.disconnected(slot);
        self.management.disconnected(slot);
        server.remove(slot);
        if child.storage.is_some() {
            server.remove(slot + 3);
        }
        world.disconnected(slot);
        // Release retained upstream image readers before returning descendant funding.
        crate::service::requests::flush(parent, world)?;
        let outcome = match child.outcome {
            Some(outcome) => outcome,
            None => {
                child.task.cancel()?;
                child.task.wait()?
            }
        };
        let state = match outcome {
            cathedral_user_runtime::task::Outcome::Returned(0) => wire::LAUNCH_EXITED,
            cathedral_user_runtime::task::Outcome::Cancelled => wire::LAUNCH_STOPPED,
            _ => wire::LAUNCH_FAILED,
        };
        if slot == 0
            && self
                .launcher
                .as_mut()
                .is_some_and(|l| !l.retired(state == wire::LAUNCH_FAILED))
        {
            self.launcher = None;
        }
        if let Some(observer) = self.observations.finished(child.task.raw(), state) {
            world.notify(observer);
        }
        if let Some(services) = child.services {
            services.close()?;
        }
        child.data.0.close()?;
        child.data.1.close()?;
        child.notify.close()?;
        if let Some((input, output)) = child.storage {
            input.close()?;
            output.close()?;
        }
        if let Some(reservation) = child.reservation {
            reservation.release()?;
            // Creator death returns descendant reservations with fresh local names.
            // Active siblings remain BUSY; static legacy grants remain DENIED.
            let mut index = 0;
            while let Ok(launch) = cathedral_user_runtime::task::Launch::at(index) {
                match launch.release() {
                    Ok(()) => (),
                    Err(Error(code)) if code == abi::BUSY as i64 || code == abi::DENIED as i64 => {
                        index += 1
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(())
    }
}
