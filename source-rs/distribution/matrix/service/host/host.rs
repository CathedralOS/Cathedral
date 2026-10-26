// Execute Omega session commands; retain resources and received events locally.
use super::*;
use cathedral_stock_matrix::policy::session::{self, Command};
pub(super) struct Host {
    parent: &'static mut Parent,
    world: &'static mut World,
    server: &'static mut Server,
    children: &'static mut children::Children,
    deferred: &'static mut deferred::Deferred,
    startup: Startup,
    ready_launch: Option<parent::Launched>,
    hold_storage: bool,
    event: Option<Event>,
    #[cfg(feature = "interaction-delay")]
    delay: delay_probe::Gate,
}
impl Host {
    pub(super) fn facts(&self, needed: u64) -> (u64, u64) {
        let mut flags = 0;
        if needed & session::BUSY != 0 && self.parent.upstream_busy() {
            flags |= session::BUSY;
        }
        if needed & session::READY_LAUNCH != 0 && self.ready_launch.is_some() {
            flags |= session::READY_LAUNCH;
        }
        if needed & session::REFRESH_READY != 0 && self.parent.refresh_ready() {
            flags |= session::REFRESH_READY;
        }
        if needed & session::PUBLISHING != 0 && self.parent.publishing() {
            flags |= session::PUBLISHING;
        }
        if needed & session::DEFERRED_READY != 0 && self.deferred.has_ready(self.parent) {
            flags |= session::DEFERRED_READY;
        }
        if needed & session::DEFERRED_EMPTY != 0 && self.deferred.is_empty() {
            flags |= session::DEFERRED_EMPTY;
        }
        if needed & session::STORING != 0 && self.parent.storing() {
            flags |= session::STORING;
        }
        if needed & session::CAN_POLL != 0 && self.parent.can_poll() {
            flags |= session::CAN_POLL;
        }
        if needed & session::HAS_WAKE != 0 && self.parent.wake.is_some() {
            flags |= session::HAS_WAKE;
        }
        if needed & session::EVENT_BLOCKED != 0
            && self
                .event
                .as_ref()
                .is_some_and(|e| deferred::Deferred::blocks(e, self.parent))
        {
            flags |= session::EVENT_BLOCKED;
        }
        let event = match &self.event {
            None => 0,
            Some(Event::Disconnected { .. }) => 1,
            Some(Event::Memory(r, _)) if r.client.is_some_and(|c| c >= PRODUCER_BASE) => 2,
            Some(Event::Memory(r, _))
                if r.client.is_some_and(|c| self.parent.storage_stream(c)) =>
            {
                3
            }
            Some(Event::Memory(_, _)) => 4,
            Some(Event::Request(_)) => 5,
        };
        (flags, event)
    }
    pub(super) fn execute(&mut self, command: Command) -> Result<bool, Error> {
        use Command::*;
        match command {
            Interaction => self.parent.progress_interaction()?,
            Refresh | RefreshRetained => self.parent.progress_refresh(self.world)?,
            Storage => self.progress_storage()?,
            AdmitReady => self
                .children
                .complete_launch(self.ready_launch.take().expect("ready launch in custody"))?,
            Binding => {
                if let Some(bound) = self.parent.progress_binding()? {
                    self.children.complete_binding(bound);
                }
            }
            Presentation => self.parent.progress_presentation(self.world, self.server)?,
            Selection => self
                .children
                .progress_selection(self.parent, self.world, self.server)?,
            LaunchScene => {
                self.children
                    .progress_launch_scene(self.parent, self.world, self.server)?
            }
            FinishChildren => {
                self.children.finish(self.server, self.world, self.parent)?;
            }
            Flush => requests::flush(self.parent, self.world)?,
            RestoreInput => self.parent.restore_input_capacity(self.world)?,
            Cursors => self.parent.cursors(self.world)?,
            Prune => self.deferred.prune(self.server),
            Launch => {
                return self
                    .children
                    .progress_launch(self.parent, self.world, self.server);
            }
            Recover => {
                return self.children.recover(self.parent, self.world, self.server);
            }
            Startup => {
                self.children
                    .startup(&mut self.startup, self.parent, self.world, self.server)?
            }
            Poll => self.parent.refresh_notified(self.world)?,
            Wait => return self.wait_event(),
            QueryLoss => {
                let handled = self.children.query_during_refresh(
                    self.event.as_ref().expect("retained event"),
                    self.parent,
                    self.server,
                )?;
                if handled {
                    self.event = None;
                }
                return Ok(handled);
            }
            Cancel => {
                let event = self.event.as_ref().expect("retained event");
                self.children.cancel_selection_event(event, self.parent);
                self.children.cancel_scene_event(event, self.parent);
            }
            Defer => self.defer_event()?,
            LegacyRefresh => self.parent.refresh(self.world)?,
            Disconnect | ProducerMemory | StorageMemory | ImageMemory | Request => {
                self.dispatch(command)?
            }
            Invalid => return Err(Error(abi::IO_ERROR as i64)),
        }
        Ok(false)
    }
}
