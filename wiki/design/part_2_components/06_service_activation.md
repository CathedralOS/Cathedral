# Chapter 06: Service Activation & Lifecycle

> A service is its endpoint capability: it exists while something that holds the capability is using it, and quiesces when idle. This chapter owns how services are named, activated on demand, supervised, and torn down, replacing both the resident daemon and the central service manager.

## The Legacy Model

A traditional OS manages services with an init system and a service manager: SysV init, launchd, the Windows Service Control Manager, and most recently systemd. The dominant pattern is the resident daemon. Services start at boot and run forever whether or not anyone uses them, so a fresh install carries dozens to hundreds of background processes. The usual defense is that an idle daemon costs no CPU, which is true and beside the point: it still holds memory (hundreds of resident sets add up to gigabytes), it is still an attack surface (a running process with standing authority and its own bugs), and it still costs power (timer wakeups and heartbeats keep the machine out of deep sleep). The cost was paid at boot, for features nobody is using.

The second failure is the service manager itself. Because spawning a process is expensive (fork, exec, load, dynamic-link, initialize a runtime), the pragmatic move is to keep daemons resident and amortize, so the daemon zoo is a symptom of costly spawn. And the manager that wires it together tends to grow. systemd began as an init replacement and absorbed logging, device management, login, name resolution, networking, timers, and mounting into one privileged, central component that everything depends on, because on a platform with soft boundaries integration is the path of least resistance. The result is a large privileged blob in the trusted base and a single coordination point for the whole system. None of this was unreasonable given expensive spawn and ambient authority; it is the honest consequence of the platform underneath.

## The Cathedral Model

A service *is* its endpoint capability. The endpoint exists as an addressable thing whether or not an instance is running behind it; holding the capability is how you reach the service, and invoking it is what brings it to life. **Access is activation**, and because it is capability-gated, you cannot even cause a service to spawn unless you hold the authority to talk to it. So a service exists only while a principal allowed to use it is using it, which removes both the resident daemon and the "any local process can poke any daemon" surface in one move.

Behind each endpoint sit two pieces of state: a **registration** (the backing component by content hash, the capability set to spawn it with, and the triggers that activate it) and a **live-instance** pointer that is empty until the service runs. Invoking the capability resolves the endpoint and then either delivers to the live instance or, on a cold miss, reads the registration, spawns the component with its declared capabilities, records the instance, and delivers. The activator is consulted only on the cold miss. A call to an already-live service is a direct capability invocation over the IPC primitive ([[ipc_and_service_invocation]]) and never touches it, so activation is not a hot-path chokepoint.

The activator is a userspace **supervisor** component rather than the kernel. The privileged core provides exactly two primitives it builds on: the trusted spawn that instantiates a component and confers its capability set, and capability invocation with its unforgeability. Everything else, the registry, the routing, the spawn-on-miss and quiesce-on-idle policy, is an ordinary component. And it is plural: the supervision tree nests ([[component_model]]), so there is no single orchestrator but a tree of small activators, each scoped to its subtree and holding only the capabilities it can hand down. None is an omnipotent init process, and none can absorb adjacent domains, because logging, networking, and the rest are separate components in their own capability domains that an activator can only spawn, never become. That is the structural answer to the systemd blob: plural, capability-bound, and mechanism-only.

Triggers are the scheduler's wait primitive ([[scheduler_and_resources]]) pointed at activation: an endpoint received a message, a device was hot-plugged ([[driver_model]]), a clock reached a time ([[time_and_clocks]]), a watched object changed ([[filesystem_as_database]]). The activator parks on these conditions and spawns the declared handler when one fires, so periodic and event-driven work needs no polling daemon, only a registered wake condition. Dependencies activate transitively for free: a service that needs another holds a capability to it, and invoking that capability activates it in turn, so there is no explicit ordering graph to declare and maintain.

The shape that results is a small **resident core** of latency-critical always-on services (the compositor, input, audio, the store, the network demux) plus a **demand-activated tail** of everything else, under an adaptive quiesce policy that keeps hot services warm and lets cold ones exit. Residency is paid only where it earns its keep. The entire advantage rests on **cheap spawn**, which the content-addressed code (already in memory, deduped), the single address space (no memory-management-unit setup or process creation for OS components), and language isolation (no ring transition) are designed to deliver. If spawn turned out expensive the model would degrade back toward keeping things warm, so cheap spawn is the load-bearing assumption, named here honestly rather than assumed.

## The decided mechanics

**The endpoint is a small dispatch object.** Invoking a service loads the endpoint's live-instance pointer and branches: live → a direct call (a virtual call, cheap); cold → the slow path, which calls the activator, `await`s the spawn, fills the pointer, and proceeds. The warm path is a plain dispatch with no kernel in the SAS; the cold path routes through the activator and the core's spawn provider — still function calls in the SAS (no ring transition), a trap when the target is hardware-walled. The unavoidable cold cost is the spawn *work*, and whether spawn is object-cheap or pays MMU domain-setup is the same SAS-vs-walled question as [[kernel_architecture]] — so the resident-core ↔ demand-tail balance shifts with the isolation substrate.

**The activator is an actor on one mailbox.** Every trigger source is normalized into a variant posted to the activator's mailbox — the core's interrupt stub turns a device interrupt into a message, the timer subsystem posts `TimerFired`, an endpoint message is already one, a watched object posts a change — so the activator does one wait on a `TriggerEvent` sum and switches to spawn the handler. There is no per-trigger task; one actor parks on its whole condition set ([[ipc_and_service_invocation]]).

**Timers are tickless.** The timer subsystem keeps registered timers ordered by expiry and arms the hardware timer as a *one-shot* at the earliest — not a periodic tick. On fire it drains all due timers (pop while `expiry ≤ now`, where firing is a cheap enqueue-and-wake into each target's mailbox) and re-arms for the next future expiry, so even a thousand timers at one instant fire in a single pass. Zero work between expirations is what lets the machine deep-sleep with no polling daemon ([[power_management]]).

**Quiesce is a cost model, not a timeout.** Weigh stay-warm cost against re-spawn cost — where re-spawn includes the power-cycle-as-consumable transition wear ([[power_management]]) and any state-rehydration cost — and enforce it as a scheduler budget, so hysteresis falls out and it composes with resource governance. A stateful service persists to the store and rehydrates on spawn (reattach-to-last-commit, [[memory_and_persistence]]); cheap-to-rehydrate state quiesces, expensive state stays resident.

**The registry is durable; live-instance pointers are soft.** Registrations — boot-seeded *plus* everything added at runtime — are persisted to the activator's realm and rehydrated on a crash-restart, so runtime registrations survive the activator's death (a boot-list alone would restore only the floor). Live-instance pointers are soft: surviving instances are reattached, or the next cold-miss re-resolves an orphan. The supervision tree bottoms out in the eagerly-booted resident core, so there is always a warm root to field the first cold-miss.

**Registration is declarative-first.** Most triggers are declared in the boot manifest and the activator registers them *for* the service, so it need not run at boot to be registered (it is spawned later when the trigger fires); a running service registers dynamically by invoking a `Capability<Activator::Register>` handed in its launch context, granted per-manifest (least authority — only services and drivers that register receive it). Predictive pre-warming to hide cold-start latency is an opt-in optimization carrying the same behavior-modeling privacy cost as predictive wake-coalescing, not a core default.

**The per-Matrix boot manifest — resolved** (mechanism in [[security_policy_and_sandboxing]], "A Matrix is an object owning its world-realm"): a **typed config object in the Matrix's own realm**, shipped in the template closure or written by the parent at provisioning, read and applied by the Matrix's mediator on boot — spawn the resident set, register the declared triggers, grant register-capabilities — and edited via the cap to it (a watched-object trigger can hot-apply changes). The residual is the exact manifest *schema*, an implementation detail discovered at dev.

## Concerns & Design Space

- **Service as an endpoint capability.** A service's identity is the capability to its endpoint, and the endpoint outlives any instance. Access is activation, capability-gated, so an unreachable service cannot be spawned ([[capability_model]], [[ipc_and_service_invocation]]).
- **The activation manifest.** A registration binds an endpoint to a backing component (by content hash), the capability set to spawn it with, and its triggers. It is typed config data ([[configuration_and_policy]]), reproducible because the component is named by hash.
- **Triggers are the wait primitive.** Endpoint message, device hot-plug, clock time, watched-object change: the activator parks on a condition and spawns on signal, so scheduled and event-driven work needs no resident poller ([[scheduler_and_resources]], [[time_and_clocks]]).
- **Cold-path route-or-spawn.** A live instance means a direct capability invocation; only a miss consults the activator, so steady-state calls never pass through it.
- **Transitive dependency activation.** A service reaches its dependencies by holding capabilities to them, and invoking those activates them in turn, so there is no explicit After/Requires ordering graph to maintain.
- **Adaptive quiesce.** Idle services quiesce or exit, and the policy must keep frequently-used ones warm and avoid thrashing a periodically-used service through spawn-quiesce-spawn, which needs hysteresis rather than a fixed timeout.
- **Stateless versus stateful activation.** A respawnable service is stateless or persists its state to the store and rehydrates on spawn ([[memory_and_persistence]]); a service that cannot cheaply rebuild its state stays resident or pays a rehydration cost on activation. Quiesce policy and persistence are coupled.
- **The activator is a confined supervisor.** It is a component, and the core gives it only the trusted spawn primitive and capability invocation. It holds only the capabilities it can grant its subtree, so both its failure and its authority are bounded ([[component_model]], [[error_model_and_recovery]]).
- **Plural and capability-bound.** The supervision tree nests, so activation is distributed across many bounded supervisors, none omnipotent, none able to absorb adjacent domains. This is the structural defense against a systemd-style central blob.
- **Capability-gated trigger registration.** Registering a trigger is itself an authority, so nothing can register "activate me on everything," a resident daemon in disguise, without holding that right. Over-eager or idle-resident activation is visible and attributable ([[observability_and_introspection]]).
- **Activation as a denial-of-service surface.** A principal that can invoke an endpoint can force spawns, so activation is rate-limited and budget-gated ([[scheduler_and_resources]]); holding the capability still does not let an attacker spawn-bomb the machine.
- **The resident core.** A small, named set of latency-critical services stays warm by policy; everything else is demand-activated. What belongs in the core is a deliberate, bounded list rather than an accretion.
- **Cheap spawn is load-bearing.** The model assumes instantiating a component is closer to creating an object than forking a process. Content-addressing, the single address space, and language isolation are what make that true, and if it fails the benefit erodes.
- **Zero value.** A zero registration is the inert null-object ([[omega_substrate]]): an endpoint with no backing spec resolves to nothing and activates nothing, so an unregistered or zeroed endpoint is a dead drop rather than a crash.

## Key Questions

- **Cold-spawn cost — resolved (rides the substrate):** warm invocation is a direct dispatch (a virtual call in the SAS); cold pays the spawn *work* plus an activator/core-spawn hop, which is function-calls in the SAS and a domain-setup trap when the spawned service is hardware-walled — so the actual number is the same SAS-vs-walled question as [[kernel_architecture]], and the resident-core/demand-tail balance shifts with it.
- **Quiesce policy — resolved:** a cost model (stay-warm vs re-spawn, the latter charging power-cycle transition wear and rehydration), enforced as a scheduler budget, so hysteresis falls out and it composes with resource governance.
- **Stateful rehydration — resolved:** persist to the store and reattach-to-last-commit; the rehydration cost is a term in the re-spawn cost, so the cost model keeps expensive-state services resident and quiesces cheap ones.
- **Resident-core membership — resolved (mechanism), parked (declaration):** a base set plus adaptive promotion of hot services; *how* the base set is declared per Matrix is the parked boot-manifest protocol.
- **Activator's table surviving restart — resolved:** the registry is durable (rehydrated from its realm), live-instance pointers are soft (reattached or re-resolved), and the root supervisor sits in the eagerly-booted resident core.

## Omega Leverage

- The activation **trigger** is the wait primitive ([[scheduler_and_resources]]); the registration's lifecycle is a **machine with states** (registered, activating, live, quiescing, dead) the supervisor inspects ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)).
- A service endpoint and its activation rights are **capabilities + domains**, so access-is-activation and capability-gated triggers reuse the whole authority model ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The activation manifest uses a numbered protocol schema and explicit codec,
  so registrations decode across artifact skew and are content-addressable
  ([protocol schemas](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)).
- The trusted spawn primitive is a **boundary provider** in the privileged core; the activator above it is ordinary proved Omega.
- Omega does not define an activation or supervision protocol; the registry, the route-or-spawn logic, and the quiesce policy are Cathedral runtime structure over Omega values.

## Open Questions

- **Predictive pre-warming — resolved (deferred):** an opt-in optimization to hide cold-start latency, carrying the same behavior-modeling privacy cost as predictive wake-coalescing; not a core default.
- **Quiesce as a scheduler budget — resolved:** yes, expressed as a budget rather than a bespoke timer, so it composes with the rest of resource governance.
- **How far down demand activation goes — resolved:** down to the eagerly-booted resident core (latency-critical services + the root supervision needed to activate everything else); the tail below is demand-activated.
- **The per-Matrix boot-manifest protocol — resolved** (a typed config object in the Matrix's realm, template-closure-shipped or parent-written, mediator-applied on boot — see above); the manifest *schema* is an implementation detail.

## Related
- [[component_model]] — the supervisor and instance kinds, and the nesting supervision tree.
- [[ipc_and_service_invocation]] — the endpoint and invocation primitive a service capability names.
- [[scheduler_and_resources]] — the wait primitive behind triggers, and the budgets that gate and quiesce.
- [[capability_model]] — service-as-capability and capability-gated activation.
- [[driver_model]] — device hot-plug as a trigger; drivers as demand-activated handlers.
- [[memory_and_persistence]] — persisting and rehydrating a quiesced service's state.
- [[configuration_and_policy]] — the activation manifest as typed config.
- [[observability_and_introspection]] — activation and idle-residency as attributable facts.
- [[power_management]] — wake-to-run triggers gated by the wakefulness capability, and maintenance-window batching.
