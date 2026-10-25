// Acquire the enclosing world's services, admit two children, then route events.
// Children receive interaction and separate scoped storage connections.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, Rect};
type World = cathedral_matrix::world::World<4>;
use cathedral_platform_contracts::{imports, matrix as wire, services};
use cathedral_user_runtime::{
    Error,
    event::Interest,
    matrix::upstream::{self, Pair},
    server::{Event, Server},
    services::Services,
    time,
};
use children::Children;
use rendering::Output;

pub fn run(mut services: Services) -> Result<(), Error> {
    services.enable_shared()?;
    cathedral_user_runtime::write(b"Cathedral Split: shared interaction broker ready\n")?;
    let interaction = services.interaction(1, 1)?;
    let storage = services.storage(1, 1)?;
    let mut server = Server::open_clients(8)?;
    server.enable_memory();
    static mut CONTROL: cathedral_user_runtime::server::SharedSlot =
        cathedral_user_runtime::server::SharedSlot::empty();
    let control = &raw mut CONTROL;
    // SAFETY: One entry owns the control slot, separately from service clients.
    server.enable_shared_control(unsafe { &mut *control })?;
    static mut CONNECTIONS: [cathedral_user_runtime::server::SharedSlot; 8] =
        [const { cathedral_user_runtime::server::SharedSlot::empty() }; 8];
    let slots = &raw mut CONNECTIONS;
    // SAFETY: One process entry lends disjoint persistent slots: two child
    // interactions, two storage streams, producer broker/data, capture broker/data.
    for (index, slot) in unsafe { &mut *slots }.iter_mut().enumerate() {
        server.enable_shared(index, slot)?;
    }
    static mut WORLD: World = World::empty();
    let arena = &raw mut WORLD;
    // SAFETY: Single-threaded process entry exclusively owns this arena.
    let world = unsafe { &mut *arena };
    let mut output = Output::new(services);
    let mut children = Children::empty();
    let mut producer = producer::Producer::empty();
    let mut capture = capture::Capture::empty();
    let mut storage = children.start(storage, &mut server)?;
    let mut survey: Option<loss::Survey> = None;
    let mut deepen = false;
    let mut pointers = cathedral_matrix::reception::Pointers::default();
    let mut refresh = true;
    loop {
        for control in children.controls.iter_mut().flatten() {
            control.progress();
        }
        storage.progress(&mut server)?;
        // Consume the wake before polling. A notification racing the poll stays
        // queued, so sleeping below cannot lose it.
        refresh |= interaction.wake.take()?;
        for _ in 0..8 {
            if !refresh || !world.input_ready() {
                break;
            }
            let snapshot = output
                .services
                .request_interaction([wire::POLL, 0, 0, 0, 0, 0, 0, 0], None)?;
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
                    } else if let Some(mut scope) = storage.idle_scope()
                        && let Err(error) =
                            capture.start(&mut scope, &mut output, world, &mut server)
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
                    } else if let Some(mut scope) = storage.idle_scope()
                        && let Err(error) =
                            producer.start(&mut scope, &mut output, world, &mut server)
                    {
                        time::storage_sample("split-producer-refused", &[error.0 as u64]);
                        output.failure = Some(b"NO ROOM");
                        output.header(false)?;
                    }
                } else if clicked
                    && survey.is_none()
                    && storage.idle()
                    && output.deepen_hit(event[3], event[4])
                {
                    if output.can_deepen()
                        // The current limit excludes the six notification
                        // allowances lent to Notes; collection returns them.
                        && cathedral_user_runtime::resources::budget()?.limits.channels >= 31
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
        output.cursors(world)?;
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
                if storage.idle() {
                    capture.stop(&mut output, world, &mut server)?;
                    producer.stop(&mut output, world, &mut server)?;
                    for slot in 0..2 {
                        storage.revoke(slot);
                        world.disconnected(slot);
                        output.flush(world)?;
                        children.stop(slot, &mut server)?;
                    }
                    match children.deepen(&mut storage, &mut server) {
                        Ok(()) => {
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
        let mut wake = [interaction.wake.interest(); 7];
        let mut count = 1;
        let mut control_deadline: Option<u64> = None;
        for (slot, control) in children.controls.iter().enumerate() {
            if let Some(control) = control {
                let pending = survey
                    .as_ref()
                    .is_some_and(|s| s.waiting & (1 << slot) != 0);
                let (interests, until) = control.wait(pending);
                for interest in interests.into_iter().flatten() {
                    wake[count] = interest;
                    count += 1;
                }
                if let Some(until) = until {
                    control_deadline = Some(control_deadline.map_or(until, |d| d.min(until)));
                }
            }
        }
        let storage_wait = storage.wait();
        if let Some((handle, _)) = storage_wait {
            wake[count] = Interest::Ipc(handle);
            count += 1;
        }
        if let Some(interest) = storage.wake() {
            wake[count] = interest;
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
        let deadline = match (deadline, storage.transfer_deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let deadline = match (deadline, control_deadline) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let Some(event) = server.next_event_for(&wake[..count], deadline)? else {
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
