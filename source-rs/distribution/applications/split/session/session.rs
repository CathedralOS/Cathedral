// Acquire the enclosing world's services, admit two children, then route events.
// Children receive interaction and separate scoped storage connections.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, Rect};
type World = cathedral_matrix::world::World<4>;
use cathedral_platform_contracts::{imports, matrix as wire, services};
use cathedral_user_runtime::{
    Error,
    matrix::upstream::{self, Pair},
    server::{Event, Server},
    services::Services,
    time,
};
use children::Children;
use rendering::Output;

pub fn run(mut services: Services) -> Result<(), Error> {
    let interaction = services.interaction(1, 1)?;
    let storage = services.storage(1, 1)?;
    let mut server = Server::open_clients(8)?;
    server.enable_memory();
    static mut WORLD: World = World::empty();
    let arena = &raw mut WORLD;
    // SAFETY: Single-threaded process entry exclusively owns this arena.
    let world = unsafe { &mut *arena };
    let mut output = Output::new(interaction.pair);
    let mut children = Children::empty();
    let mut producer = producer::Producer::empty();
    let mut capture = capture::Capture::empty();
    let mut storage = children.start(storage, &mut server)?;
    let mut survey: Option<loss::Survey> = None;
    let mut deepen = false;
    let mut pointers = cathedral_matrix::reception::Pointers::default();
    let mut refresh = true;
    loop {
        storage.progress(&mut server)?;
        // Consume the wake before polling. A notification racing the poll stays
        // queued, so sleeping below cannot lose it.
        match interaction.wake.receive_until(&mut [0; 64], time::now()?) {
            Ok(_) => refresh = true,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(e) => return Err(e),
        }
        for _ in 0..8 {
            if !refresh || !world.input_ready() {
                break;
            }
            let snapshot = upstream::call(interaction.pair, [wire::POLL, 0, 0, 0, 0, 0, 0, 0])?;
            let grant = world.inherit_poll(snapshot);
            output.arrange(grant, world)?;
            producer.maintain(&mut output, world, &mut server)?;
            let event = wire::poll_event(snapshot);
            let clicked = pointers.update(event).is_some_and(|b| b.left_pressed());
            if event[2] == wire::POINTER {
                if clicked
                    && event[4] < 28
                    && event[3] >= grant.width.saturating_sub(28)
                    && survey.is_none()
                {
                    deepen = false;
                    survey = Some(loss::Survey::start(None, &mut children)?);
                } else if clicked && survey.is_none() && output.capture_hit(event[3], event[4]) {
                    if capture.running() {
                        capture.stop(&mut output, world, &mut server)?;
                    } else if let Some(pair) = storage.idle_pair()
                        && let Err(error) = capture.start(pair, &mut output, world, &mut server)
                    {
                        time::storage_sample("split-capture-refused", &[error.0 as u64]);
                        output.failure = Some(b"CANNOT CAPTURE");
                    }
                    output.header(false)?;
                } else if clicked && survey.is_none() && output.visibility_hit(event[3], event[4]) {
                    output.toggle_right(world)?;
                } else if clicked && survey.is_none() && output.automatic_hit(event[3], event[4]) {
                    if producer.running() {
                        producer.stop(&mut output, world, &mut server)?;
                        output.header(false)?;
                    } else if let Some(pair) = storage.idle_pair()
                        && let Err(error) = producer.start(pair, &mut output, world, &mut server)
                    {
                        time::storage_sample("split-producer-refused", &[error.0 as u64]);
                        output.failure = Some(b"NO ROOM");
                        output.header(false)?;
                    }
                } else if clicked
                    && survey.is_none()
                    && storage.idle_pair().is_some()
                    && output.deepen_hit(event[3], event[4])
                {
                    if output.can_deepen()
                        && cathedral_user_runtime::resources::budget()?.limits.channels >= 15
                    {
                        deepen = true;
                        survey = Some(loss::Survey::start(None, &mut children)?);
                    } else {
                        output.failure = Some(b"NO ROOM");
                        output.header(false)?;
                    }
                }
            }
            refresh = snapshot[5] & wire::MORE != 0;
        }
        producer.maintain(&mut output, world, &mut server)?;
        capture.progress(&mut output, world, &mut server)?;
        output.flush(world)?;
        children.notify(world)?;
        if let Some(s) = &mut survey
            && let Some(clean) = s.progress(&mut children)?
        {
            if let Some(request) = s.request {
                server.reply(
                    &request,
                    &wire::encode([0, u64::from(!clean), 0, 0, 0, 0, 0, 0]),
                )?;
            } else if clean && deepen {
                // Descendants answered clean; collect their grants and images
                // before lending the same conserved funding to the new matrix.
                if let Some(pair) = storage.idle_pair() {
                    capture.stop(&mut output, world, &mut server)?;
                    producer.stop(&mut output, world, &mut server)?;
                    for slot in 0..2 {
                        storage.revoke(slot);
                        world.disconnected(slot);
                        output.flush(world)?;
                        children.stop(slot, &mut server)?;
                    }
                    match children.deepen(pair, &mut server) {
                        Ok(next) => {
                            storage = next;
                            output.nested_layout(world)?;
                        }
                        Err(error) => {
                            cathedral_user_runtime::time::storage_sample(
                                "split-nesting-refused",
                                &[error.0 as u64],
                            );
                            output.failure = Some(b"CANNOT OPEN");
                            output.header(false)?;
                        }
                    }
                }
                deepen = false;
            } else if clean {
                capture.stop(&mut output, world, &mut server)?;
                producer.stop(&mut output, world, &mut server)?;
                return Ok(());
            } else {
                output.header(true)?;
            }
            survey = None;
        }
        if refresh && world.input_ready() {
            continue;
        }
        let mut wake = [interaction.wake; 4];
        let mut count = 1;
        if let Some(s) = &survey {
            for slot in 0..2 {
                if s.waiting & (1 << slot) != 0 {
                    wake[count] = children.slots[slot].as_ref().unwrap().control.1;
                    count += 1;
                }
            }
        }
        let storage_wait = storage.wait();
        if let Some((handle, _)) = storage_wait {
            wake[count] = handle;
            count += 1;
        }
        server.pause(
            u32::from(survey.as_ref().is_some_and(|s| s.request.is_some()))
                | if storage_wait.is_some() {
                    (1 << 3) | (1 << 4)
                } else {
                    0
                },
        );
        let deadline = match (
            survey.as_ref().map(|s| s.deadline),
            storage_wait.map(|(_, deadline)| deadline),
        ) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let deadline = match (deadline, capture.deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let Some(event) = server.next_event_or(&wake[..count], deadline)? else {
            continue;
        };
        let client = match &event {
            Event::Request(r) | Event::Memory(r, _) => r.client,
            Event::Disconnected { client, .. } => Some(*client),
        };
        if client.is_some_and(|c| c >= capture::BROKER) {
            capture.dispatch(event, &mut output, world, &mut server)?;
            continue;
        }
        if client.is_some_and(|c| c >= producer::BROKER) {
            producer.dispatch(event, &mut output, world, &mut server)?;
            continue;
        }
        requests::dispatch(
            event,
            &mut output,
            world,
            &mut children,
            &mut server,
            &mut survey,
            &mut storage,
        )?;
        // Pin/render a source frame before dispatching any later publication.
        capture.progress(&mut output, world, &mut server)?;
    }
}
