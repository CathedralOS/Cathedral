//! Host mechanisms requested by the Omega launch workflow.
use super::*;
impl Children {
    pub(super) fn launch_step(
        &mut self,
        command: Command,
        l: &mut Pending,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<bool, Error> {
        let slot = l.plan.slot;
        #[cfg(any(feature = "launch-bind-failure", feature = "launch-place-failure"))]
        if l.probe
            && command
                == if cfg!(feature = "launch-bind-failure") {
                    Command::Bind
                } else {
                    Command::Place
                }
        {
            time::storage_sample("launch-stage-failure", &[command as u64]);
            return Err(Error(abi::IO_ERROR as i64));
        }
        match command {
            Command::RetirePicker => {
                self.stop(0, server, world, parent)?;
                // The selected workspace receives the final viewport from admission onward.
                self.standing[slot].as_mut().unwrap().surfaces = world.backed_surfaces() as u64;
            }
            Command::Admit => {
                let mut image = l.image.take().expect("matching read completion");
                let result = self.run_object(l.plan, &mut image, parent, server, world);
                drop(image);
                match result {
                    Ok(storage) => {
                        l.task = self.slots[slot].as_ref().map(|c| c.task.raw());
                        parent.admitting = Some(slot);
                        #[cfg(feature = "trace-storage")]
                        time::storage_sample(
                            "stored-launch-admitted",
                            &[slot as u64, l.ticket.raw()],
                        );
                        return Ok(storage);
                    }
                    Err(e) => return Err(e),
                }
            }
            Command::Bind => parent.begin_storage_binding(l.ticket, slot)?,
            Command::Place => {
                if let Some(standing) = self.standing[slot] {
                    let range = if self.desktop {
                        self.place_surface_range(
                            world,
                            slot,
                            self.requested_surfaces[slot].unwrap_or(10) as usize,
                        )
                    } else if self.login {
                        world.surface_range(slot, 0, standing.surfaces as usize)
                    } else {
                        surface_limit(world, slot, standing.surfaces as usize)
                    };
                    range.map_err(|c| Error(c as i64))?;
                    if let Some((rect, keys)) = standing.place {
                        world.place(slot, rect, keys).map_err(|c| Error(c as i64))?;
                    }
                }
            }
            Command::Activate => {
                l.previous = self.active;
                l.previous_task = self
                    .active
                    .and_then(|s| self.slots[s].as_ref())
                    .map(|c| c.task.raw());
                parent.begin_scene(
                    l.ticket,
                    world,
                    self.active,
                    Some(slot),
                    slot == 2 && !cfg!(feature = "selection-delay"),
                )?;
            }
            Command::Publish => {
                // Admitted-child requests stay held through this command. No
                // child surface exists yet, so publication needs no replay IPC.
                debug_assert!(
                    (0..wire::MAX_SURFACES)
                        .all(|rank| world.preview_replay(slot, rank, true).is_none())
                );
                self.notify(world)?;
                #[cfg(feature = "trace-storage")]
                time::storage_sample("stored-launch-published", &[slot as u64, l.ticket.raw()]);
                if l.selection {
                    self.entered = true;
                    cathedral_user_runtime::write(
                        b"Cathedral matrix: picker retired; user owns viewport\n",
                    )?;
                }
            }
            Command::Rollback => {
                self.restore_launch_scene(l, parent, world, server)?;
                // Never reclaim a replacement task that happens to occupy the same slot.
                if self.slots[slot].as_ref().map(|c| c.task.raw()) == l.task && l.task.is_some() {
                    self.stop(slot, server, world, parent)?;
                }
                l.task = None;
            }
            Command::RestorePicker => {
                self.entered = false;
                self.startup_recovery
                    .policy
                    .selection_failed(self.startup_recovery.owner_recovers);
            }
            _ => unreachable!("read and finish are handled by the launch entrance"),
        }
        Ok(false)
    }
}
