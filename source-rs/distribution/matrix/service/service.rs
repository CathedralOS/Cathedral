// Consume local invalidations, wake children, then dispatch ownership and input.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, Rect};
const CHILDREN: usize = 5;
const LAST_CHILD: u64 = CHILDREN as u64 - 1;
const SOURCE: usize = CHILDREN;
const STORAGE_BASE: usize = CHILDREN + 1;
const PRODUCER_BASE: usize = STORAGE_BASE + CHILDREN;
const STREAMS: usize = PRODUCER_BASE + CHILDREN + 1;
const BACKGROUND: usize = 4;
type World = cathedral_matrix::world::World<CHILDREN>;
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    event::Interest,
    server::{Event, Server},
    task::Environment,
};
use parent::Parent;

/// Stock sibling stacking: slot zero below slot one, regardless of launch order.
fn surface_limit(world: &mut World, slot: usize, count: usize) -> Result<(), u64> {
    let base = match slot {
        0 => 0,
        1 => world
            .backed_surfaces()
            .checked_sub(count)
            .ok_or(abi::NO_MEMORY)?,
        _ => return Err(abi::DENIED),
    };
    world.surface_range(slot, base, count)
}

/// Whether this matrix still has to read its realm for what to become. The
/// root adapter and owner-driven fixtures are configured by commands instead.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Startup {
    Hosted,
    Realm,
}
pub fn run(environment: Environment) -> Result<(), Error> {
    // Supplied provider links make this instance the storage root; nothing else does.
    let storage_root = cathedral_user_runtime::link::Link::at(0).is_ok();
    static mut PARENT: Parent = Parent::empty();
    let parent_address = &raw mut PARENT;
    // SAFETY: Single-threaded executable enters once. Persistent buffers live in
    // process-owned data pages, not constructor temporaries on the guarded stack.
    let parent = unsafe { &mut *parent_address };
    parent.initialize(&environment, storage_root)?;
    let mut startup = if parent.upstream {
        Startup::Realm
    } else {
        Startup::Hosted
    };
    static mut WORLD: World = World::empty();
    let world_address = &raw mut WORLD;
    // SAFETY: Single-threaded executable enters this service exactly once. This
    // process-owned arena has no aliases and never occupies the guarded user stack.
    let world = unsafe { &mut *world_address };
    let mut server = Server::open_clients_from(if storage_root { 2 } else { 0 }, STREAMS - 1)?;
    server.enable_memory();
    server.isolate_control_failures();
    static mut CONTROL: cathedral_user_runtime::server::SharedSlot =
        cathedral_user_runtime::server::SharedSlot::empty();
    let control = &raw mut CONTROL;
    // SAFETY: This process lends its owner-control storage once, until exit.
    server.enable_shared_control(unsafe { &mut *control })?;
    static mut REPLIES: cathedral_user_runtime::server::Replies =
        cathedral_user_runtime::server::Replies::empty();
    let replies_address = &raw mut REPLIES;
    // SAFETY: This single-entry service owns the static bounded reply arena.
    server.retain_replies(unsafe { &mut *replies_address });
    if environment.len() == 4 {
        server.install(
            SOURCE,
            (
                environment.endpoint(2).unwrap(),
                environment.endpoint(3).unwrap(),
            ),
        )?;
        static mut SHARED: cathedral_user_runtime::server::SharedSlot =
            cathedral_user_runtime::server::SharedSlot::empty();
        let slot = &raw mut SHARED;
        // SAFETY: Single-entry service owns this connection storage until exit.
        server.enable_shared(SOURCE, unsafe { &mut *slot })?;
    }
    if parent.separate_storage {
        static mut INTERACTION: [cathedral_user_runtime::server::SharedSlot; CHILDREN] =
            [const { cathedral_user_runtime::server::SharedSlot::empty() }; CHILDREN];
        let slots = &raw mut INTERACTION;
        // SAFETY: One service entry owns these disjoint child connection slots.
        for (index, slot) in unsafe { &mut *slots }.iter_mut().enumerate() {
            server.enable_pipeline(index, slot)?;
        }
        static mut STORAGE: [cathedral_user_runtime::server::SharedSlot; CHILDREN] =
            [const { cathedral_user_runtime::server::SharedSlot::empty() }; CHILDREN];
        let slots = &raw mut STORAGE;
        // SAFETY: One service entry lends disjoint persistent slots once. Removing
        // a child resets custody; the replacement keeps this protocol selection.
        for (index, slot) in unsafe { &mut *slots }.iter_mut().enumerate() {
            server.enable_shared(STORAGE_BASE + index, slot)?;
        }
    }
    // Page allowances and use at startup, so a refused attachment is diagnosable.
    if let Ok(budget) = cathedral_user_runtime::resources::budget() {
        cathedral_user_runtime::time::storage_sample(
            "matrix-budget",
            &[
                budget.limits.private_pages,
                budget.limits.reader_pages,
                budget.used.private_pages,
                budget.used.reader_pages,
            ],
        );
    }
    // Two stamps back to back: their gap is what one trace line itself costs,
    // which hop_latency.py subtracts from every stage it reports.
    #[cfg(feature = "trace-storage")]
    for _ in 0..2 {
        cathedral_user_runtime::time::storage_trace("trace-calibrate", 0, 0);
    }
    let mut children = children::Children::default();
    children.separate_storage = parent.separate_storage;
    static mut DEFERRED: deferred::Deferred = deferred::Deferred::empty();
    let deferred_address = &raw mut DEFERRED;
    // SAFETY: Same single-entry process lifetime as the parent/world arenas above.
    let deferred = unsafe { &mut *deferred_address };
    let mut ready_launch = None;
    #[cfg(feature = "interaction-delay")]
    let mut delay = delay_probe::Gate::default();
    loop {
        parent.progress_interaction()?;
        parent.progress_refresh(world)?;
        // The test-only probe can count this matrix's storage as busy, as a
        // slow provider would, around a close, so its answer is seen to wait.
        #[cfg(feature = "interaction-delay")]
        let hold_storage = {
            if core::mem::take(&mut parent.hold_storage_requested) {
                parent.hold_storage_armed = true;
                cathedral_user_runtime::time::storage_sample(
                    "interaction-delay-armed",
                    &[wire::STORAGE_BASE],
                );
            }
            let now = cathedral_user_runtime::time::now()?;
            let holding = parent
                .hold_storage_until
                .is_some_and(|until| !cathedral_user_runtime::time::reached(now, until));
            if parent.hold_storage_active && !holding {
                parent.hold_storage_until = None;
                cathedral_user_runtime::time::storage_sample("interaction-delay-released", &[]);
                children.release_held_loss();
            }
            parent.hold_storage_active = holding;
            holding
        };
        #[cfg(not(feature = "interaction-delay"))]
        let hold_storage = false;
        if !hold_storage && let Some(launched) = parent.progress_storage(&mut server)? {
            assert!(ready_launch.is_none());
            ready_launch = Some(launched);
        }
        // Storage progresses independently. Admission waits for interaction
        // ownership, rather than consuming an outstanding snapshot's reply.
        if !parent.upstream_busy()
            && let Some(launched) = ready_launch.take()
        {
            children.complete_launch(launched)?;
        }
        if let Some(bound) = parent.progress_binding()? {
            children.complete_binding(bound);
        }
        parent.progress_presentation(world, &mut server)?;
        children.progress_selection(parent, world, &mut server)?;
        children.progress_launch_scene(parent, world, &server)?;
        // A replacement snapshot retained behind publication is already in
        // memory: consume it now, rather than parking on an empty receiver.
        if parent.refresh_ready() && !parent.publishing() {
            parent.progress_refresh(world)?;
        }
        children.finish(&mut server, world, parent)?;
        if !parent.upstream_busy() {
            requests::flush(parent, world)?;
            parent.restore_input_capacity(world)?;
            parent.cursors(world)?;
        }
        deferred.prune(&server);
        if !deferred.has_ready(parent) && children.progress_launch(parent, world, &mut server)? {
            continue;
        }
        if deferred.is_empty() && children.recover(parent, world, &mut server)? {
            continue;
        }
        if !parent.storing() && !parent.upstream_busy() {
            children.startup(&mut startup, parent, world, &mut server)?;
        }
        // Ready mutations get their turn before another poll. A queue waiting
        // on independent storage must not prevent consuming interaction wakes.
        if parent.can_poll() && !deferred.has_ready(parent) {
            parent.refresh_notified(world)?;
        }
        let (interests, mut deadline) = children.wait(
            parent.upstream_busy() || parent.storing(),
            cathedral_user_runtime::time::now()?,
        );
        children.notify(world)?;
        let mut extra =
            [Interest::Ipc(cathedral_user_runtime::ipc::Handle::from_raw(0)); CHILDREN * 2 + 6];
        let mut count = 0;
        if !hold_storage && let Some((receive, until)) = parent.storage_wait() {
            extra[count] = Interest::Ipc(receive);
            count += 1;
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
        if let Some(until) = parent.recovery_delay_deadline() {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        #[cfg(feature = "binding-delay")]
        if let Some(until) = parent.binding_delay_deadline() {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        for (receive, until) in [
            parent.refresh_wait(),
            parent.presentation_wait(),
            parent.scene_wait(),
        ]
        .into_iter()
        .flatten()
        {
            // Both exchanges may wait on the same interaction receiver.
            if !extra[..count]
                .iter()
                .any(|h| matches!(h,Interest::Ipc(old) if old.raw()==receive.raw()))
            {
                extra[count] = Interest::Ipc(receive);
                count += 1;
            }
            if let Some(until) = until {
                deadline = Some(deadline.map_or(until, |d| {
                    if cathedral_user_runtime::time::reached(d, until) {
                        until
                    } else {
                        d
                    }
                }));
            }
        }
        if !hold_storage {
            if let Some(interest) = parent.storage_wake() {
                extra[count] = interest;
                count += 1;
            }
            if let Some(until) = parent.storage_transfer_deadline() {
                deadline = Some(deadline.map_or(until, |d| {
                    if cathedral_user_runtime::time::reached(d, until) {
                        until
                    } else {
                        d
                    }
                }));
            }
        }
        if let Some(interest) = parent.output_wake()? {
            extra[count] = interest;
            count += 1;
        }
        if let Some(until) = parent.output_deadline() {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        if let Some(until) = parent.poll_retry {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        for interest in interests.into_iter().flatten() {
            extra[count] = interest;
            count += 1;
        }
        let input_ready = world.input_ready();
        if parent.can_poll() && parent.refresh_due && input_ready {
            deadline = Some(cathedral_user_runtime::time::now()?);
        }
        #[cfg(feature = "interaction-delay")]
        for until in delay
            .deadline()
            .into_iter()
            .chain(parent.hold_storage_until)
        {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        if parent.can_poll()
            && input_ready
            && let Some(wake) = parent.wake
        {
            extra[count] = wake.interest();
            count += 1;
        }
        #[cfg(not(feature = "interaction-delay"))]
        let queued = deferred.take(parent);
        #[cfg(feature = "interaction-delay")]
        let queued = delay.take(&server)?.or_else(|| deferred.take(parent));
        deferred.pause(&mut server, parent);
        #[cfg(feature = "trace-storage")]
        let from_queue = queued.is_some();
        let event = match queued {
            Some(event) => event,
            None => match server.next_event_for(&extra[..count], deadline)? {
                Some(event) => event,
                // An upstream reply or a child's wake made an extra handle
                // ready; the next turn's progress steps take it from there.
                None => continue,
            },
        };
        #[cfg(feature = "interaction-delay")]
        let event = if !parent.upstream {
            match delay.intercept(event, &mut server, world)? {
                Some(event) => event,
                None => continue,
            }
        } else {
            event
        };
        #[cfg(feature = "trace-storage")]
        if !from_queue
            && let Event::Request(r) = &event
            && r.client.is_some_and(|c| parent.storage_stream(c))
        {
            let op = wire::decode(&r.bytes[..r.len]).map_or(0, |w| w[0] & 0xffff);
            cathedral_user_runtime::time::storage_trace(
                if parent.upstream {
                    "nested-receive"
                } else {
                    "root-receive"
                },
                r.client.unwrap() as u64,
                op,
            );
        }
        // A read-only loss survey uses child control links, not the occupied
        // upstream poll stream. Do not race a completed but unadmitted launch.
        if ready_launch.is_none() && children.query_during_refresh(&event, parent, &mut server)? {
            continue;
        }
        children.cancel_selection_event(&event, parent);
        children.cancel_scene_event(&event, parent);
        if deferred::Deferred::blocks(&event, parent) {
            #[cfg(feature = "trace-storage")]
            deferred::trace_loss(&event, parent);
            #[cfg(feature = "interaction-delay")]
            if parent.refresh_late()
                && let Event::Request(r) = &event
                && wire::decode(&r.bytes[..r.len])
                    .is_some_and(|w| w[0] == wire::POLL_TAGGED || w[0] == wire::POLL)
            {
                cathedral_user_runtime::time::storage_sample("matrix-poll-local", &[]);
            }
            let mut blocked = parent.admitting.map_or(0, |slot| 1 << slot);
            if parent.storing()
                && let Some(client) = parent.storing_client()
            {
                blocked |= 1 << client;
            }
            deferred.hold(event, &mut server, world, blocked)?;
            continue;
        }
        // Notification-driven input ran at the start of this turn. Do not
        // drain it again ahead of an already-selected child request: recursive
        // polling can otherwise starve that request under continuous movement.
        // Manually assembled two-endpoint fixtures still refresh on demand.
        if !parent.upstream_busy() && parent.wake.is_none() {
            parent.refresh(world)?;
        }
        match event {
            Event::Disconnected {
                client,
                incarnation,
            } => {
                // A delayed disconnect must not tear down a replacement connection.
                if server
                    .incarnation(client)
                    .is_some_and(|current| current != incarnation)
                {
                    continue;
                }
                if client < CHILDREN {
                    children.disconnected(client, &mut server, world, parent)?;
                } else if client == SOURCE {
                    requests::source::cancel(world);
                } else if client >= PRODUCER_BASE {
                    children.producer_disconnected(
                        client - PRODUCER_BASE,
                        incarnation,
                        world,
                        &mut server,
                    )?;
                } else {
                    parent.storage_close(parent.storage_slot(client))?;
                }
            }
            Event::Memory(request, object) => {
                if request.client.is_some_and(|client| client >= PRODUCER_BASE) {
                    drop(object);
                    children.producer_request(&request, None, world, &mut server)?;
                } else if request
                    .client
                    .is_some_and(|client| parent.storage_stream(client))
                {
                    requests::attachment(request, object, parent, &mut server)?;
                } else {
                    requests::image(request, object, parent, world, &mut server)?;
                }
            }
            Event::Request(request) => {
                requests::dispatch(request, parent, world, &mut children, &mut server)?
            }
        }
    }
}
