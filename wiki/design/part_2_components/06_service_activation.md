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

- What is the actual cost of a cold spawn for a typical OS component, and what latency does a first invocation add on the interactive path?
- What quiesce policy avoids thrashing a periodically-used service while not keeping cold services warm, and how much hysteresis or prediction does it need?
- How does a stateful service persist and rehydrate cheaply enough that quiescing it beats keeping it resident?
- Which services genuinely belong in the always-warm resident core, and who decides?
- How does the activator's own routing table survive its restart, given the activator is itself a supervised component?

## Omega Leverage

- The activation **trigger** is the wait primitive ([[scheduler_and_resources]]); the registration's lifecycle is a **machine with states** (registered, activating, live, quiescing, dead) the supervisor inspects ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md), [states](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)).
- A service endpoint and its activation rights are **capabilities + domains**, so access-is-activation and capability-gated triggers reuse the whole authority model ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- The activation manifest is **wire data** with stable field numbers, so registrations decode across versions and are content-addressable ([wire protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md)).
- The trusted spawn primitive is a **boundary provider** in the privileged core; the activator above it is ordinary proved Omega.
- Omega does not define an activation or supervision protocol; the registry, the route-or-spawn logic, and the quiesce policy are Cathedral runtime structure over Omega values.

## Open Questions

- Is there a predictive or pre-warming activation that hides cold-start latency for interactive services without drifting back to resident daemons?
- Can the quiesce policy be expressed as a budget the scheduler enforces rather than a bespoke timer, so it composes with the rest of resource governance?
- How far down does demand activation go: are core services themselves lazily brought up as their first caller needs them, or eagerly started at boot?

## Related
- [[component_model]] — the supervisor and instance kinds, and the nesting supervision tree.
- [[ipc_and_service_invocation]] — the endpoint and invocation primitive a service capability names.
- [[scheduler_and_resources]] — the wait primitive behind triggers, and the budgets that gate and quiesce.
- [[capability_model]] — service-as-capability and capability-gated activation.
- [[driver_model]] — device hot-plug as a trigger; drivers as demand-activated handlers.
- [[memory_and_persistence]] — persisting and rehydrating a quiesced service's state.
- [[configuration_and_policy]] — the activation manifest as typed config.
- [[observability_and_introspection]] — activation and idle-residency as attributable facts.
