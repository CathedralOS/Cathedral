# Chapter 01: Scheduler & Resource Governance

> Every finite resource is authority. This chapter owns CPU, memory, IO, energy and the rest as *budgeted capabilities*, declared by intent and enforced by the OS.

## The Legacy Model

A traditional scheduler governs one thing well — CPU time — and the rest badly or not at all. Memory is handed out until the OOM killer fires a blunt, surprising shot. IO, network, GPU, wakeups, battery, thermal headroom, and storage-write endurance are governed by a scatter of unrelated mechanisms: `nice`, cgroups, `ionice`, QoS classes, wake locks, throttling daemons. None of it is unified, and almost none of it is *authority*: a process that can run can, by default, also allocate memory, write the disk, wake the device, and burn the battery. Resource abuse is ambient power that nobody had to grant.

## The Cathedral Model

Two moves. First, components **declare their resource behavior as intent**, in a vocabulary the OS understands:

```omega
data ResourceIntent {
    class:    WorkloadClass;   // latency_sensitive | interactive_ui |
                               // realtime_audio | background_sync |
                               // batch_compute | power_saving
    deadline: Option<Duration>;
    priority: LatencyPriority;
}
```

Second, the OS **enforces budgets**, and crucially: *resource rights are capabilities too.* You cannot write storage without a storage-write budget; you cannot run in the background without a budget that the power and network policy admit; you cannot allocate beyond your memory envelope.

```omega
Capability<Cpu::Budget(share, deadline_class)>
Capability<Memory::Budget(working_set_max)>
Capability<Storage::WriteBudget(bytes_per_window)>
Capability<Background::Run(power_policy)>
Capability<Gpu::Budget(compute_share)>
Capability<Accelerator::Budget(npu_share)>
```

This folds resource governance into the one authority model ([[capability_model]]): the scheduler is not a separate subsystem with its own ad-hoc rules, it is the *enforcement arm* of resource capabilities.

### Tasks and the one wait primitive

The schedulable unit is the **task**: a running state machine. An instance is one or more tasks; the scheduler sees tasks, while budgets and accounting attach to the instance and component.

A task runs until it must wait, and the scheduler offers exactly one way to wait: **park the task until a condition is signaled, then unpark it.** A condition is one of three things:

- a shared word reaches a value (the producer/consumer case from [[ipc_and_service_invocation]]),
- a channel changes state (a message arrived, or space freed),
- a clock reaches a time ([[time_and_clocks]]).

Everything that blocks reduces to this. A sleep or a timer is parking on a clock condition. A blocking receive is parking on a channel condition. A device interrupt is the device unparking a driver task ([[driver_model]]). There is no separate timer subsystem and no separate blocking-IPC subsystem; they are one primitive with different conditions. The adaptive default is spin-then-park: spin briefly to catch the common case with no scheduler round-trip, then park if the condition has not arrived.

Park does not just resume — it returns a **wake reason**, because the thing a task waits on can also *die*. A parked task must wake on more than its happy condition, or it sleeps forever on a corpse:

- **Signaled** — the condition was met (word reached, message arrived, time came). The normal case.
- **PeerDied** — a holder of the thing being waited on is gone. This is the one liveness fact only the OS can supply, sourced from the grant arena ([[capability_lifecycle]]): the waited-on region or endpoint is an object with tracked capability-holders, and a holder's death drops its arena entry, which fires this wake to anyone parked on it. A blocking receive on a dead producer returns `PeerDied`, not an eternal sleep.
- **Revoked** — the capability backing the waited-on object was revoked (the mapped-grant teardown, [[capability_lifecycle]]); the straggler wakes here rather than faulting where it can.
- **Timeout** — a deadline on the park elapsed.

Liveness is therefore not a separate subsystem and not a header polled out of the shared page (which a hostile peer could forge); it is the arena and the scheduler meeting at the wake. The arena knows who holds a region; the scheduler delivers death as one reason among the normal ones. A spinning task that never parks reads the same fact from an OS-owned, peer-read-only status word instead — push when parked, pull when spinning, same truth.

There are no locks to wait on, because mutual exclusion is ownership, enforced at compile time ([[component_model]]): two tasks cannot hold a mutable reference to the same data, so there is nothing to lock. A task only ever waits for a value, a message, or a time.

## Concerns & Design Space

- **Capability-gated resource access.** A held budget is required to *touch* a resource, not just to be prioritized within it. Absence of budget = absence of the effect, audited like any capability.
- **Per-component budgets & accounting.** Every instance ([[component_model]]) is a billable entity; the OS can always answer "what is this spending."
- **Heterogeneous compute.** The CPU is one of several engines. The NPU, GPU, and DPU are first-class schedulable, isolated, budgeted resources, multiplexed like any shared device ([[driver_model]]). An agent's inference ([[agents_as_principals]]) is scheduled and metered on the NPU exactly like CPU time on the CPU.
- **Deadline & realtime-ish scheduling.** `realtime_audio` and `interactive_ui` need bounded latency without true hard-RT guarantees most apps can't honor.
- **Park/unpark on a condition.** The one blocking primitive: a task parks until a word, channel, or clock condition is signaled, and the signaler unparks it. Park returns a **wake reason** (`Signaled` / `PeerDied` / `Revoked` / `Timeout`), so a parked task wakes on the death or revocation of what it waits on, not only its happy condition — `PeerDied` and `Revoked` are sourced from the grant arena's region membership ([[capability_lifecycle]]), the one liveness fact only the OS can supply. Spin-then-park is the default policy; pure spinning is reserved for dedicated cores and reads the same death fact from an OS-owned read-only status word ([[ipc_and_service_invocation]], [[time_and_clocks]]).
- **Priority inversion.** A high-priority component blocked on a low-priority one holding a resource — inheritance/donation must be modeled, not accidental.
- **Backpressure propagation.** When a downstream service is budget-saturated, pressure must flow upstream as a typed signal the caller can act on ([[error_model_and_recovery]], [[ipc_and_service_invocation]]).
- **Task storage is a proven bound, not an allocation.** A parked task is data, not a stack ([Omega concurrency](../../../../Omega/wiki/language_guide/chapter_17_concurrency.md)): per-machine-type pools of `M × N`, where M is the compiler-computed carry-set and **N is derived from the finite resource the task parks on** (a single-consumer mailbox → 1; a permit/budget pool → its capacity; a connection table → its rows), never a magic constant. So total task memory is a compile-time bound and spawning never OOMs — a wrong N fails a model-checked invariant at design time, not in production. This is the fd-table generalized: one budgeted handle table for tasks as for grants ([[capability_lifecycle]]), with no global pool to exhaust. The carry-set is single-level (Omega forbids suspend-in-call) and collapses to the actor's own `self` under the run-to-completion pattern; co-locating it in the bounding resource's slot avoids a separate pool entirely. Static pools are the bounded default; an allocator-backed dynamic pool is the explicit, type-visible escape for genuinely unbounded N, trading the static-footprint proof for flexibility.
- **Memory pressure & OOM policy.** Replace the OOM killer with a negotiated, policy-driven reclaim: components declare shrinkable caches, the OS asks before it takes.
- **Energy & thermal accounting.** Per-component energy attribution and thermal budget as first-class, surfaced to the user ([[power_management]], [[observability_and_introspection]]).
- **Fairness across tenants.** Budgets must compose hierarchically so one tenant cannot starve another ([[component_model]] tenant axis).
- **Nested budgets.** Budgets sub-allocate down the tree: a component with a budget hands attenuated slices to its children and can never give more than it holds, so hierarchical fair-share and per-VM or per-container limits are the recursive-provider pattern ([[capability_model]]) applied to resources.
- **Abuse prevention.** A component cannot escalate its own budget; only a broker can mint or widen one ([[capability_lifecycle]]).
- **Zero value.** A zero executor domain inherits the parent's envelope and a zero reservation means best-effort rather than zero share, so the zero `ResourceIntent` is the inherit-or-default shape (shape 3): a deliberate choice that lets a spawned task run under ambient defaults instead of failing for want of an explicit budget ([[omega_substrate]]).

## Key Questions

- What is the canonical resource vocabulary, and is it open (drivers add new resource kinds) or fixed?
- Is a budget a hard ceiling, a weighted share, a reservation, or all three depending on resource kind?
- How does declared intent reconcile with measured behavior — does the OS demote a component that lies about being `power_saving`?
- Where do deadlines come from, and who is trusted to assert `realtime_audio`?
- How are accelerators (NPU/GPU/DPU) scheduled and isolated across components: the same multiplexed-queue pattern as the NIC ([[networking]], [[driver_model]]), and is preemption even possible on them?

## Omega Leverage

- Resource rights as **capabilities (values + domains)** — `Storage::Writable` with a budget domain — reuse the whole authority machinery ([../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **`effects`** already name the resource-touching boundaries (`filesystem_io`, `device_io`, `network_io`); a budget capability is the runtime gate on each.
- A component's lifecycle **`state` graph** exposes natural scheduling points — the scheduler can see *which state* an instance is in, not just that it's runnable.
- Omega does **not** model resource *quantities* or deadlines as proof obligations; budget arithmetic, accounting, and enforcement are Cathedral runtime — a candidate for bounded-value contracts to grow into.

## Open Questions

- Can budgets be expressed tightly enough to *prove* a realtime component meets its deadline, or is that always runtime best-effort?
- How are budgets reclaimed on crash or revocation without a window where a resource leaks?
- Is global energy/thermal a true shared budget all components draw from, and how is that arbitrated fairly under contention?

## Related
- [[capability_model]] — resource rights are capabilities in the same graph.
- [[component_model]] — the instance as the unit of budget and accounting.
- [[power_management]] — energy, thermal, and wakefulness as governed resources.
- [[observability_and_introspection]] — resource use as a queryable surface.
