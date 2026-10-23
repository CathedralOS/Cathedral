//! Resolve a child launch through its own realm, then admit, place and observe it.
use super::*;
impl Children {
    /// A child's request for a sibling. The other slot must be free and opened
    /// by the owner; the program is read with the requester's own scope, so a
    /// launcher can only launch what it can see.
    pub(in crate::service) fn launch(
        &mut self,
        request: Request,
        object: u64,
        withhold: u64,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.loss.is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let requester = request.client.unwrap();
        let slot = if self.desktop {
            if !self.chrome(&request, world) {
                return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
            }
            let Some(slot) = (1..BACKGROUND).find(|s| self.slots[*s].is_none()) else {
                time::storage_sample("funding-local-slots-refused", &[3]);
                return server.reply(&request, &wire::encode(wire::error(abi::NO_MEMORY)));
            };
            slot
        } else if requester < 2 {
            1 - requester
        } else {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        };
        let refusal = if parent.storing() || self.slots[slot].is_some() || !server.available(slot) {
            Some(abi::BUSY)
        } else if !self.observations.available(requester) || self.next_ticket == u64::MAX {
            Some(abi::NO_MEMORY)
        } else if object == 0 || withhold & !cathedral_platform_contracts::imports::ALL != 0 {
            Some(abi::INVALID_ARGUMENT)
        } else {
            None
        };
        if let Some(code) = refusal {
            return server.reply(&request, &wire::encode(wire::error(code)));
        }
        let (Some(standing), Some((_, stamp))) = (
            self.standing[slot],
            parent.storage_binding(server, requester),
        ) else {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        };
        parent.storage_launch(
            request,
            parent::LaunchRequest {
                slot,
                object,
                scope: object,
                argument: 0,
                channels: standing.channels,
                descendants: standing.descendants,
                cpu: standing.cpu,
                pages: Some(standing.pages),
                stamp,
                imports: self.allowed_imports[slot]
                    .unwrap_or(cathedral_platform_contracts::imports::ALL)
                    & !withhold,
            },
        )
    }
    /// Admit a program read from the matrix's realm and answer whoever asked.
    /// Selected storage binds the child to that object's subtree; a failed
    /// binding stops the child. Optional storage may instead be absent.
    /// A sibling launch also gets the slot's standing surfaces and placement.
    pub(in crate::service) fn launch_object(
        &mut self,
        launched: parent::Launched,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let parent::Launched {
            plan,
            request,
            result,
        } = launched;
        let Some(request) = request else {
            if let Some(launcher) = &mut self.launcher {
                launcher.result = Some(result);
            }
            return Ok(());
        };
        let mut status = match result {
            Err(code) => Err(code),
            Ok(mut object) => {
                if self.login && !self.entered && plan.slot == 1 {
                    // Reuse the picker account for the workspace. Keeping it
                    // alive through reservation strands that lower slot outside
                    // the workspace's descendant pool. Code is already loaded.
                    self.stop(0, server, world, parent)?;
                }
                let admitted = self.run_object(plan, &mut object, parent, server, world);
                drop(object); // The kernel holds its own reference to the image.
                match admitted {
                    Err(error) => Err(error.0 as u64),
                    Ok(false) => Ok(()),
                    Ok(true) => match parent.storage_grant(server, plan.slot, 0, plan.scope)? {
                        Ok(()) => {
                            self.saved_storage[plan.slot] = Some((0, plan.scope));
                            Ok(())
                        }
                        Err(code) => {
                            self.stop(plan.slot, server, world, parent)?;
                            Err(code)
                        }
                    },
                }
            }
        };
        if let Err(code) = status {
            time::storage_sample("stored-launch-refused", &[plan.slot as u64, code]);
        }
        // Retire the picker before publishing the user's first view. Otherwise
        // a workspace can configure chrome against a temporary smaller allowance.
        if status.is_ok() && self.login && !self.entered && plan.slot == 1 {
            self.entered = true;
            self.standing[1].as_mut().unwrap().surfaces = world.backed_surfaces() as u64;
            cathedral_user_runtime::write(
                b"Cathedral matrix: picker retired; user owns viewport\n",
            )?;
        }
        if status.is_err() && self.login && !self.entered && self.slots[0].is_none() {
            self.startup_recovery.picker_pending = true;
            self.startup_recovery.selection_failed = true;
            self.startup_recovery.picker_attempts = 0;
        }
        if status.is_ok()
            && request.client.is_some()
            && let Some(standing) = self.standing[plan.slot]
        {
            let placement = (if self.desktop {
                self.place_surface_range(
                    world,
                    plan.slot,
                    self.requested_surfaces[plan.slot].unwrap_or(10) as usize,
                )
            } else if self.login {
                world.surface_range(plan.slot, 0, standing.surfaces as usize)
            } else {
                surface_limit(world, plan.slot, standing.surfaces as usize)
            })
            .and_then(|()| {
                match standing.place {
                    Some((rect, keys)) => world.place(plan.slot, rect, keys),
                    None => Ok(()), // Headless policy deliberately has no placement.
                }
            });
            if let Err(code) = placement {
                self.stop(plan.slot, server, world, parent)?;
                status = Err(code);
            }
        }
        if status.is_err()
            && self.login
            && plan.slot == 1
            && self.slots[1].is_none()
            && self.slots[0].is_none()
        {
            self.entered = false;
            self.startup_recovery.picker_pending = true;
            self.startup_recovery.selection_failed = true;
            self.startup_recovery.picker_attempts = 0;
        }
        if status.is_ok() && self.desktop && request.client == Some(0) {
            self.next_ticket += 1;
            self.running[plan.slot] = Some((self.next_ticket, plan.object));
            let code = self.activate(plan.slot, parent, world, server)?;
            if code != 0 {
                self.stop(plan.slot, server, world, parent)?;
                status = Err(code);
            }
        }
        super::requests::flush(parent, world)?;
        super::requests::replay(parent, world, plan.slot)?;
        self.notify(world)?;
        // The owner named the slot; a child learns which one it got.
        let reply = match (status, request.client) {
            (Ok(()), Some(observer)) => {
                let receipt = self
                    .observations
                    .record(
                        observer,
                        request.incarnation,
                        self.slots[plan.slot].as_ref().unwrap().task.raw(),
                    )
                    .expect("checked receipt capacity");
                [0, plan.slot as u64, receipt, 0, 0, 0, 0, 0]
            }
            (Ok(()), None) => [0; 8],
            (Err(code), _) => wire::error(code),
        };
        server.reply(&request, &wire::encode(reply))
    }
}
