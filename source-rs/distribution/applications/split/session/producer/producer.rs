// Split's explicit demonstration policy: one extra participant, one private
// focus group and one owner-funded cursor. View loss ends this run; AUTO starts
// a new process/grant. Protocol dispatch is in requests.rs, admission beside it.
use super::*;
use cathedral_user_runtime::{
    matrix::admission::{self as spawn, Admitted},
    task::Launch,
};
pub const BROKER: usize = 4;
pub const INPUT: usize = 5;
const SLOT: usize = 2;
pub struct Producer {
    child: Option<Admitted>,
    launch: Option<Launch>,
    cursor: Option<(u64, u64)>,
    point: Option<(usize, u64, u64)>,
    painted: Option<(u64, u64, bool)>,
    limited: Option<[usize; 2]>,
}
impl Producer {
    pub fn empty() -> Self {
        Self {
            child: None,
            launch: None,
            cursor: None,
            point: None,
            painted: None,
            limited: None,
        }
    }
    pub fn running(&self) -> bool {
        self.child.is_some()
    }
    pub fn start(
        &mut self,
        scope: &mut cathedral_user_runtime::storage::Scope<'_>,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let result = self.admit(scope, output, world, server);
        if result.is_err() {
            self.stop(output, world, server)?;
        }
        result
    }
    pub fn stop(
        &mut self,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        world.disconnected(SLOT);
        self.point = None;
        self.painted = None;
        server.remove(BROKER);
        server.remove(INPUT);
        if let Some(child) = self.child.take() {
            child.task.cancel()?;
            child.task.wait()?;
            child.close_connections()?;
        }
        if let Some(launch) = self.launch.take() {
            launch.release()?;
        }
        if let Some((epoch, cursor)) = self.cursor.take() {
            let reply = output
                .services
                .request_interaction([wire::ERASE, epoch, cursor, 0, 0, 0, 0, 0], None)?;
            if !matches!(reply[0], 0 | abi::DENIED | abi::BAD_HANDLE) {
                return Err(Error(reply[0] as i64));
            }
        }
        if output.automatic {
            output.automatic = false;
            cathedral_user_runtime::write(
                b"Cathedral Split: participant stopped and cursor retired\n",
            )?;
        }
        Ok(())
    }
    pub fn maintain(
        &mut self,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if let Some(child) = &self.child {
            let info = world.request(
                SLOT,
                child.data.0.raw(),
                [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0],
            );
            if info[0] != 0 {
                self.stop(output, world, server)?;
                output.header(false)?;
            }
        }
        if self.limited.is_some() && output.grant.valid() && world.input_ready() {
            let capacity = world
                .inherited_input_headroom(0, 0)
                .map_err(|c| Error(c as i64))?;
            if self.limited != Some(capacity) {
                let reply = output.services.request_interaction(
                    [
                        wire::INPUT_CAPACITY,
                        output.grant.epoch,
                        capacity[0] as u64,
                        capacity[1] as u64,
                        0,
                        0,
                        0,
                        0,
                    ],
                    None,
                )?;
                if reply[0] == 0 {
                    self.limited = Some(capacity);
                } else if !matches!(reply[0], abi::DENIED | abi::BUSY) {
                    return Err(Error(reply[0] as i64));
                }
            }
            if self.limited == Some([wire::INPUT_STREAMS, wire::INPUT_GROUPS]) {
                self.limited = None;
            }
        }
        if let Err(error) = self.render_cursor(output, world) {
            if matches!(error.0 as u64, abi::DENIED | abi::BAD_HANDLE) {
                self.stop(output, world, server)?;
                output.header(false)?;
            } else {
                return Err(error);
            }
        }
        Ok(())
    }
}
