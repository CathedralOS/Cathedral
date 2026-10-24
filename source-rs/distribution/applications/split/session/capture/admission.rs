//! Source and crop are owner policy; the consumer cannot choose siblings or masks.
use super::*;
use cathedral_user_runtime::{resources::Limits, storage::Scope, task::Executable};
impl Capture {
    pub(super) fn admit(
        &mut self,
        pair: Pair,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.running()
            || output.nested
            || output.capacity < 11
            || output.grant.width < 420
            || output.grant.height < 300
        {
            return Err(Error(abi::NO_MEMORY as i64));
        }
        let incarnation = server
            .incarnation(SOURCE)
            .ok_or(Error(abi::DENIED as i64))?;
        let view = world.request(SOURCE, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        if view[0] != 0 {
            return Err(Error(view[0] as i64));
        }
        let plan = Plan::new(
            view[2],
            view[3],
            Crop {
                x: 0,
                y: 0,
                width: 200,
                height: 104,
            },
            &[Crop {
                x: 10,
                y: 20,
                width: 180,
                height: 64,
            }],
        )
        .map_err(|_| Error(abi::INVALID_ARGUMENT as i64))?;
        let mut scope = Scope::new(pair);
        scope.bind().map_err(|c| Error(c as i64))?;
        let id = scope.lookup(b"Capture").map_err(|c| Error(c as i64))?;
        let (code, granted) =
            children::image::read_offered(&mut scope, id, imports::INTERACTION | imports::CAPTURE)?;
        let launch = Executable::from_object(&code, 0)?.reserve(
            Limits {
                // The viewer retains a sealed reader and allocates no private pixels.
                // Keep output funding in this provider until the masked image is frozen.
                private_pages: 0,
                reader_pages: 32,
                channels: 0,
            },
            0,
            abi::MAX_ADMISSION_FRAMES as u64,
        )?;
        self.launch = Some(launch);
        self.child = Some(spawn::spawn_brokered(launch, 0, &[], granted)?);
        let child = self.child.as_ref().unwrap();
        server.install(BROKER, child.data)?;
        server.install(ENDPOINT, child.capture.unwrap())?;
        world
            .surface_range(SLOT, 10, 1)
            .map_err(|c| Error(c as i64))?;
        world
            .place(
                SLOT,
                Rect {
                    x: 12,
                    y: 160,
                    width: 200,
                    height: 104,
                },
                0,
            )
            .map_err(|c| Error(c as i64))?;
        world.request(SLOT, child.data.0.raw(), [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        let source = Source {
            participant: Participant {
                slot: SOURCE,
                incarnation,
            },
            epoch: view[1],
        };
        let receiver = Participant {
            slot: ENDPOINT,
            incarnation: child.capture.unwrap().0.raw(),
        };
        self.lease
            .grant(source, receiver, plan)
            .map_err(|_| Error(abi::DENIED as i64))?;
        self.source = Some(source);
        self.receiver = Some(receiver);
        self.parent = output.grant;
        output.capturing = true;
        cathedral_user_runtime::write(b"Cathedral capture: explicit crop grant admitted\n")?;
        Ok(())
    }
}
