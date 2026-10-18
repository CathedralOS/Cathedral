//! One local retry for the launcher selected by CONFIGURE. Never rerun the
//! configuration or disturb the app slot. Code reads yield through the ordinary
//! storage state machine, and replacement uses fresh task/endpoint grants.
use super::*;
use cathedral_matrix_service::world::LauncherPlacement;
use cathedral_user_runtime::memory::Object;

pub(super) struct Launcher {
    plan: parent::LaunchRequest,
    task: u64,
    incarnation: u64,
    snapshot: Option<(LauncherPlacement, [u64; 4])>,
    pub retired: bool,
    reading: bool,
    pub result: Option<Result<Object, u64>>,
}
impl Launcher {
    pub fn new(plan: parent::LaunchRequest, task: u64, incarnation: u64) -> Self {
        Self {
            plan,
            task,
            incarnation,
            snapshot: None,
            retired: false,
            reading: false,
            result: None,
        }
    }
}
impl Children {
    /// Snapshot before disconnect erases the dead child's view. A pending
    /// control reply may defer final collection, so capture only once.
    pub(super) fn capture_recovery(&mut self, slot: usize, world: &World, parent: &Parent) {
        if slot != 0 {
            return;
        }
        let Some(launcher) = &mut self.launcher else {
            return;
        };
        if launcher.snapshot.is_some() {
            return;
        }
        if self.slots[0].as_ref().map(|child| child.task.raw()) != Some(launcher.task)
            || self.saved_storage[0] != Some((0, 0))
        {
            self.launcher = None;
            return;
        }
        match (
            world.launcher_placement(launcher.incarnation),
            parent.storage_identity(),
        ) {
            (Some(placement), Some(binding)) => launcher.snapshot = Some((placement, binding)),
            _ => self.launcher = None,
        }
    }
    /// A successful explicit owner mutation supersedes CONFIGURE's recovery
    /// policy. In particular, revocation cannot be undone by a queued retry.
    pub(in crate::service) fn recovery_owner_changed(&mut self, words: wire::Packet) {
        if words[0] == wire::BIND
            || (words[1] == 0
                && matches!(
                    words[0],
                    wire::PLACE
                        | wire::REVOKE
                        | wire::SURFACE_LIMIT
                        | wire::STORAGE_GRANT
                        | wire::STORAGE_REVOKE
                        | wire::LAUNCH_IMPORTS
                        | wire::RUN_PAGES
                        | wire::RUN_MEMORY
                        | wire::LAUNCH_POLICY
                        | wire::LAUNCH_PLACE
                ))
        {
            self.launcher = None;
        }
    }
    pub(in crate::service) fn recovery_pending(&self) -> bool {
        self.launcher
            .as_ref()
            .is_some_and(|launcher| launcher.retired)
    }
    /// True means progress was made; loop before sleeping to send the first
    /// storage request or serve the newly admitted launcher's initial requests.
    pub(in crate::service) fn recover(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<bool, Error> {
        if !self.recovery_pending()
            || parent.upstream_busy()
            || parent.storing()
            || self.loss.is_some()
            || self.pending.iter().any(Option::is_some)
        {
            return Ok(false);
        }
        parent.refresh(world)?;
        let launcher = self.launcher.as_ref().unwrap();
        let (placement, binding) = launcher.snapshot.expect("captured before retirement");
        if !world.launcher_parent_matches(placement)
            || parent.storage_identity() != Some(binding)
            || self.slots[0].is_some()
            || !server.available(0)
        {
            self.launcher = None;
            return Ok(false);
        }
        if !launcher.reading {
            parent.storage_recover(launcher.plan)?;
            self.launcher.as_mut().unwrap().reading = true;
            return Ok(true);
        }
        if launcher.result.is_none() {
            return Ok(false);
        }
        // Consume the single retry before admission, including failed admission.
        // A second crash cannot schedule another retry or retain the old image.
        let launcher = self.launcher.take().unwrap();
        let status = match launcher.result.unwrap() {
            Err(code) => Err(code),
            Ok(mut object) => {
                let admitted = self.run_object(launcher.plan, &mut object, server, world);
                drop(object);
                match admitted {
                    Err(error) => Err(error.0 as u64),
                    Ok(false) => Err(abi::DENIED),
                    Ok(true) => match parent.storage_grant(server, 0, 0, 0)? {
                        Err(code) => Err(code),
                        Ok(()) if parent.storage_identity() != Some(binding) => Err(abi::DENIED),
                        Ok(()) => world.restore_launcher(placement, server.incarnation(0).unwrap()),
                    },
                }
            }
        };
        if let Err(code) = status {
            if self.slots[0].is_some() {
                self.stop(0, server, world, parent)?;
            }
            time::storage_sample("launcher-recovery-refused", &[code]);
        } else {
            self.saved_storage[0] = Some((0, 0));
            self.notify(world)?;
            cathedral_user_runtime::write(
                b"Cathedral matrix: launcher recovered; retry consumed\n",
            )?;
        }
        Ok(true)
    }
}
