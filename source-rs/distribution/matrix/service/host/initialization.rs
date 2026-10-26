//! Install supplied connections and process-owned storage for the Omega session.
use super::*;
impl Host {
    pub(in crate::service) fn initialize(environment: Environment) -> Result<Self, Error> {
        // Supplied provider links make this instance the storage root; nothing else does.
        let storage_root = cathedral_user_runtime::link::Link::at(0).is_ok();
        static mut PARENT: Parent = Parent::empty();
        let parent_address = &raw mut PARENT;
        // SAFETY: Single-threaded executable enters once. Persistent buffers live in
        // process-owned data pages, not constructor temporaries on the guarded stack.
        let parent = unsafe { &mut *parent_address };
        parent.initialize(&environment, storage_root)?;
        let startup = if parent.upstream {
            Startup::Realm
        } else {
            Startup::Hosted
        };
        static mut WORLD: World = World::empty();
        let world_address = &raw mut WORLD;
        // SAFETY: Single-threaded executable enters this service exactly once. This
        // process-owned arena has no aliases and never occupies the guarded user stack.
        let world = unsafe { &mut *world_address };
        static mut SERVER: core::mem::MaybeUninit<Server> = core::mem::MaybeUninit::uninit();
        let server_address = &raw mut SERVER;
        // SAFETY: Single entry constructs the server before publishing its sole
        // mutable reference. Moving the whole host through the stack duplicates
        // large server/child records; their arenas instead live for this process.
        let server = unsafe {
            (*server_address).write(Server::open_clients_from(
                if storage_root { 2 } else { 0 },
                STREAMS - 1,
            )?)
        };
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
        {
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
            static mut PRODUCER: [cathedral_user_runtime::server::SharedSlot; CHILDREN] =
                [const { cathedral_user_runtime::server::SharedSlot::empty() }; CHILDREN];
            let slots = &raw mut PRODUCER;
            // SAFETY: Disjoint producer connections have one process-owned arena each.
            for (index, slot) in unsafe { &mut *slots }.iter_mut().enumerate() {
                server.enable_shared(PRODUCER_BASE + index, slot)?;
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
        static mut CHILD_STATE: core::mem::MaybeUninit<children::Children> =
            core::mem::MaybeUninit::uninit();
        let children_address = &raw mut CHILD_STATE;
        // SAFETY: Initialized exactly once; the service owns the only reference.
        let children = unsafe { (*children_address).write(children::Children::default()) };
        static mut DEFERRED: deferred::Deferred = deferred::Deferred::empty();
        let deferred_address = &raw mut DEFERRED;
        // SAFETY: Same single-entry process lifetime as the parent/world arenas above.
        let deferred = unsafe { &mut *deferred_address };
        let ready_launch = None;
        #[cfg(feature = "interaction-delay")]
        let delay = delay_probe::Gate::default();
        Ok(Self {
            parent,
            world,
            server,
            children,
            deferred,
            startup,
            ready_launch,
            hold_storage: false,
            event: None,
            #[cfg(feature = "interaction-delay")]
            delay,
        })
    }
}
