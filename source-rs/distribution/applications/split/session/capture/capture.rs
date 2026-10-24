// An explicitly admitted consumer sees one masked crop of the right Notes.
// Admission selects authority; requests authenticates IPC; progress pins the
// acknowledged source scene before another publication can replace its readers.
use super::*;
use cathedral_matrix::capture::{Lease, Participant, Plan, Rect as Crop, Source};
use cathedral_platform_contracts::capture as wire_capture;
use cathedral_user_runtime::{
    matrix::admission::{self as spawn, Admitted},
    memory::{Object, Private},
    task::Launch,
};
pub const BROKER: usize = 6;
pub const ENDPOINT: usize = 7;
const SLOT: usize = 3;
const SOURCE: usize = 1;
const LIFETIME: u64 = 300;
struct Pending {
    ticket: u64,
    frame: u64,
    deadline: u64,
    target: Private,
}
pub struct Capture {
    child: Option<Admitted>,
    launch: Option<Launch>,
    source: Option<Source>,
    receiver: Option<Participant>,
    parent: Grant,
    lease: Lease<Object>,
    pending: Option<Pending>,
    failure: Option<(u64, u64)>,
}
impl Capture {
    pub fn empty() -> Self {
        Self {
            child: None,
            launch: None,
            source: None,
            receiver: None,
            parent: Grant::default(),
            lease: Lease::empty(),
            pending: None,
            failure: None,
        }
    }
    pub fn running(&self) -> bool {
        self.child.is_some()
    }
    pub fn deadline(&self) -> Option<u64> {
        self.pending.as_ref().map(|p| p.deadline)
    }
    pub fn start(
        &mut self,
        pair: Pair,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let result = self.admit(pair, output, world, server);
        if result.is_err() {
            self.stop(output, world, server)?;
        }
        result
    }
    fn release_frame(&mut self, world: &mut World) {
        if let Some(p) = self.pending.take()
            && let Some(s) = self.source
        {
            let _ = world.release_frame(
                s.participant.slot,
                s.participant.incarnation,
                s.epoch,
                p.frame,
            );
            let _ = cathedral_user_runtime::write(
                b"Cathedral capture: pending frame and output released\n",
            );
        }
    }
    pub fn stop(
        &mut self,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        self.release_frame(world);
        self.lease.revoke();
        self.source = None;
        self.receiver = None;
        self.failure = None;
        world.disconnected(SLOT);
        output.flush(world)?;
        server.remove(BROKER);
        server.remove(ENDPOINT);
        if let Some(child) = self.child.take() {
            child.task.cancel()?;
            child.task.wait()?;
            child.close_connections()?;
        }
        if let Some(launch) = self.launch.take() {
            launch.release()?;
        }
        if output.capturing {
            output.capturing = false;
            cathedral_user_runtime::write(
                b"Cathedral capture: consumer and retained result retired\n",
            )?;
        }
        Ok(())
    }
    pub fn progress(
        &mut self,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(source) = self.source else {
            return Ok(());
        };
        if output.grant != self.parent
            || server.incarnation(SOURCE) != Some(source.participant.incarnation)
            || !world.binding_matches(SOURCE, source.participant.incarnation, source.epoch)
        {
            self.stop(output, world, server)?;
            output.header(false)?;
            return Ok(());
        }
        let Some(pending) = &self.pending else {
            return Ok(());
        };
        let now = time::now()?;
        self.lease.advance(now);
        if time::reached(now, pending.deadline) {
            self.release_frame(world);
            cathedral_user_runtime::write(
                b"Cathedral capture: request expired; source retained\n",
            )?;
            return Ok(());
        }
        if !world.frame_ready(
            SOURCE,
            source.participant.incarnation,
            source.epoch,
            pending.frame,
        ) {
            return Ok(());
        }
        let mut pending = self.pending.take().unwrap();
        let plan = *self.lease.plan().unwrap();
        let rendered = (|| {
            output.capture(world, source, &plan, &mut pending.target)?;
            let mut object = pending.target.freeze()?;
            object.unmap()?;
            Ok::<_, Error>(object)
        })();
        let _ = world.release_frame(
            SOURCE,
            source.participant.incarnation,
            source.epoch,
            pending.frame,
        );
        match rendered {
            Ok(object) => {
                if self
                    .lease
                    .publish(source, pending.ticket, time::now()?, object)
                    .is_err()
                {
                    self.failure = Some((pending.ticket, wire_capture::EXPIRED));
                }
            }
            Err(error) => {
                self.lease
                    .release(self.receiver.unwrap(), pending.ticket)
                    .ok();
                self.failure = Some((pending.ticket, error.0 as u64));
            }
        }
        Ok(())
    }
}
