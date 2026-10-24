//! Negotiate inherited capacity before reserving local production. Unwind every
//! partially installed resource through Producer::stop; no implicit regrant.
use super::*;
use cathedral_user_runtime::{resources::Limits, storage::Scope, task::Executable};
impl Producer {
    pub(super) fn admit(
        &mut self,
        pair: Pair,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.running() || output.nested || output.capacity < 10 || !world.input_ready() {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        let mut scope = Scope::new(pair);
        scope.bind().map_err(|c| Error(c as i64))?;
        let id = scope.lookup(b"Producer").map_err(|c| Error(c as i64))?;
        let (code, offered) = children::image::read_offered(
            &mut scope,
            id,
            imports::INTERACTION | imports::INPUT_PRODUCER,
        )?;
        let capacity = world
            .inherited_input_headroom(2, 1)
            .map_err(|c| Error(c as i64))?;
        if capacity[0] < 2 {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        let grant = output.grant;
        let reply = upstream::call(
            output.pair,
            [
                wire::INPUT_CAPACITY,
                grant.epoch,
                capacity[0] as u64,
                capacity[1] as u64,
                0,
                0,
                0,
                0,
            ],
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        self.limited = Some(capacity);
        let reply = upstream::call(
            output.pair,
            [wire::SURFACE_CREATE, grant.epoch, 9, 0, 0, 0, 0, 0],
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        self.cursor = Some((grant.epoch, reply[1]));
        let launch = Executable::from_object(&code, 0)?.reserve(
            Limits {
                private_pages: 16,
                reader_pages: 0,
                channels: 0,
            },
            0,
            abi::MAX_ADMISSION_FRAMES as u64,
        )?;
        self.launch = Some(launch);
        let child = spawn::spawn_brokered(launch, 0, &[], offered)?;
        let incarnation = child.data.0.raw();
        self.child = Some(child);
        let child = self.child.as_ref().unwrap();
        server.install(BROKER, child.data)?;
        server.install(INPUT, child.producer.unwrap())?;
        // This non-rendering participant is never placed over either document.
        world
            .place(
                SLOT,
                Rect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                0,
            )
            .map_err(|c| Error(c as i64))?;
        world.request(SLOT, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        world
            .grant_input_producer(SLOT, 2, wire::PRODUCER_FOCUS, capacity[0], capacity[1])
            .map_err(|c| Error(c as i64))?;
        output.automatic = true;
        output.header(false)?;
        cathedral_user_runtime::write(
            b"Cathedral Split: participant admitted with scoped input and cursor\n",
        )?;
        Ok(())
    }
}
