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
            parent.batches[slot].reset();
            world.disconnected(slot);
            return Ok(());
        }
        if self.slots[slot].is_some() {
            self.stop(slot, server, world, parent)?;
        } else {
            parent.batches[slot].reset();
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
        // Authorized stop quiesces the task before withdrawing its services.
        // Upstream erasure can park us: a still-running child must not observe
        // half-removed bindings and fault before we get back to cancellation.
        let outcome = match child.outcome {
            Some(outcome) => outcome,
            None => {
                child.task.cancel()?;
                child.task.wait()?
            }
        };
        self.saved_storage[slot] = None;
        self.catalogue.retire(slot);
        if self.desktop && slot != 0 {
            world.notify(0);
        }
        parent.grant_recovery(slot, None);
        self.observations.disconnected(slot);
        self.management.disconnected(slot);
        server.remove(slot);
        if child.storage.is_some() {
            server.remove(slot + STORAGE_BASE);
        }
        if child.producer.is_some() {
            server.remove(slot + PRODUCER_BASE);
        }
        world.disconnected(slot);
        // Release retained upstream image readers before returning descendant funding.
        crate::service::requests::flush(parent, world)?;
        let state = match outcome {
            cathedral_user_runtime::task::Outcome::Returned(0) => wire::LAUNCH_EXITED,
            cathedral_user_runtime::task::Outcome::Cancelled => wire::LAUNCH_STOPPED,
            _ => wire::LAUNCH_FAILED,
        };
        if let cathedral_user_runtime::task::Outcome::Fault {
            vector,
            error,
            address,
            instruction,
        } = outcome
        {
            time::storage_sample(
                "matrix-child-fault",
                &[slot as u64, vector, error, address, instruction],
            );
        }
        self.startup_retired(slot, state == wire::LAUNCH_FAILED);
        #[cfg(feature = "trace-storage")]
        if slot == 0 && self.launcher.is_some() {
            time::storage_sample("launcher-retired", &[state]);
        }
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
        // Shared interaction may have closed this exact incarnation on removal.
        for endpoint in [child.data.0, child.data.1] {
            match endpoint.close() {
                Ok(()) => (),
                Err(Error(c)) if c == abi::BAD_HANDLE as i64 => (),
                Err(error) => return Err(error),
            }
        }
        child.notify.close()?;
        if let Some((input, output)) = child.storage {
            // Shared setup owns and may already have closed this pair during
            // disconnect/removal. Never reinterpret a retired local handle.
            for endpoint in [input, output] {
                match endpoint.close() {
                    Ok(()) => (),
                    Err(Error(c)) if c == abi::BAD_HANDLE as i64 => (),
                    Err(error) => return Err(error),
                }
            }
        }
        if let Some((input, output)) = child.producer {
            // Shared teardown may already have closed this exact incarnation.
            for endpoint in [input, output] {
                match endpoint.close() {
                    Ok(()) => (),
                    Err(Error(c)) if c == abi::BAD_HANDLE as i64 => (),
                    Err(error) => return Err(error),
                }
            }
        }
        drop(child.control);
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
        if self.desktop
            && let Some(next) = self.catalogue.fallback()
        {
            self.activate(next, parent, world, server)?;
        }
        // The kernel's view of this account once the subtree is gone: pages
        // and frames still charged and launch slots free again. A soak compares
        // these across cycles, so a leak shows as drift rather than as time.
        #[cfg(feature = "trace-storage")]
        {
            let budget = cathedral_user_runtime::resources::budget()?;
            let physical = cathedral_user_runtime::resources::physical_budget()?;
            time::storage_sample(
                "matrix-reclaimed",
                &[
                    slot as u64,
                    budget.used.private_pages,
                    budget.used.reader_pages,
                    physical.charged,
                    cathedral_user_runtime::task::available_slots()?,
                ],
            );
        }
        Ok(())
    }
}

impl Children {
    /// A child's request to stop its sibling. The sibling is asked first and
    /// kept if it reports work it would lose or does not answer.
    pub(in crate::service) fn close(
        &mut self,
        request: Request,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.loss.is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let target = if self.desktop {
            if !self.chrome(&request, world) {
                return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
            }
            let Some(target) = self.catalogue.active() else {
                return server.reply(&request, &wire::encode(wire::error(abi::PEER_CLOSED)));
            };
            target
        } else if request.client.unwrap() < 2 {
            1 - request.client.unwrap()
        } else {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        };
        // A reply the requester sent just before asking may still be queued.
        self.finish(server, world, parent)?;
        if self.pending[target].is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let Some(child) = self.slots[target].as_mut() else {
            return server.reply(&request, &wire::encode(wire::error(abi::PEER_CLOSED)));
        };
        #[cfg(feature = "interaction-delay")]
        if core::mem::take(&mut parent.hold_storage_armed) {
            // Test-only: this close counts the matrix's storage as busy for
            // two seconds, longer than its answer's deadline, and the sibling
            // learns of it only when that ends, so the clean answer lands
            // after the busy window, inside the grace the held deadline keeps.
            parent.hold_storage_until = Some(time::after(200)?);
            parent.hold_storage_active = true;
            self.held_loss[target] = true;
            time::storage_sample("interaction-delay-held", &[wire::LOSS]);
            self.pending[target] = Some((request, time::after(100)?, Purpose::Close));
            self.retiring[target] = None;
            return Ok(());
        }
        if child
            .control
            .ask([wire::LOSS, 0, 0, 0, 0, 0, 0, 0])
            .is_err()
        {
            // A closed control channel alone is not permission to discard work.
            if self.completed(target)? {
                self.stop(target, server, world, parent)?;
                return server.reply(&request, &wire::encode([0; 8]));
            }
            #[cfg(feature = "trace-storage")]
            time::storage_sample("matrix-close-unavailable", &[target as u64]);
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        self.pending[target] = Some((request, time::after(100)?, Purpose::Close));
        #[cfg(feature = "trace-storage")]
        time::storage_trace("matrix-close-send", request.incarnation, target as u64);
        self.retiring[target] = None;
        Ok(())
    }
    /// Test-only: send the LOSS a close held back while the storage hold was
    /// active. Its answer's deadline has been running since the close itself.
    #[cfg(feature = "interaction-delay")]
    pub(in crate::service) fn release_held_loss(&mut self) {
        for slot in 0..CHILDREN {
            if !core::mem::take(&mut self.held_loss[slot]) {
                continue;
            }
            if let Some(child) = &mut self.slots[slot] {
                // A failed send leaves the pending close to time out as it would.
                child.control.ask([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]).ok();
            }
        }
    }
}
