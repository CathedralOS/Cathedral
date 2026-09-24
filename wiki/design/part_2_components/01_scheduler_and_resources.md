# Chapter 01: Scheduler & Resource Governance

> Every finite resource is authority. CPU, memory, IO, energy, and the rest are budgeted capabilities, declared by intent and enforced by the OS.

## The Legacy Model

A traditional scheduler governs CPU time well and everything else badly or not at all. Memory is handed out until the OOM killer fires, late and unpredictably. IO, network, GPU, wakeups, battery, thermal headroom, and storage-write endurance are governed by a scatter of unrelated mechanisms: `nice`, cgroups, `ionice`, QoS classes, wake locks, throttling daemons. None of it is unified, and almost none of it is authority. A process that can run can, by default, also allocate memory, write the disk, wake the device, and burn the battery. Resource abuse is ambient power that nobody had to grant.

## The Cathedral Model

Cathedral makes two moves. First, components declare their resource behavior as intent, in a vocabulary the OS understands:

```omega
data ResourceIntent {
    class:    WorkloadClass;   // latency_sensitive | interactive_ui |
                               // realtime_audio | background_sync |
                               // batch_compute | power_saving
    deadline: Optional<Duration>;
    priority: LatencyPriority;
}
```

Second, the OS enforces **budgets**, and resource rights are capabilities. You cannot write storage without a storage-write budget. You cannot run in the background without a budget that the power and network policy admit. You cannot allocate beyond your memory envelope.

```omega
Capability<Cpu::Budget(share, deadline_class)>
Capability<Memory::Budget(working_set_max)>
Capability<Storage::WriteBudget(bytes_per_window)>
Capability<Background::Run(power_policy)>
Capability<Gpu::Budget(compute_share)>
Capability<Accelerator::Budget(npu_share)>
```

This folds resource governance into the one authority model ([[capability_model]]). The scheduler is not a separate subsystem with its own rules. It is the enforcement arm of resource capabilities.

### Tasks and the one wait primitive

The schedulable unit is the task: a running state machine. An instance is one or more tasks. The scheduler sees tasks, while budgets and accounting attach to the instance and component.

A runnable task may be timer-preempted and resumed at any instruction. When it must wait, the scheduler offers one wait primitive: park the task until a condition is signaled, then unpark it. A condition is one of three things:

- a shared word reaches a value (the producer/consumer case from [[ipc_and_service_invocation]]),
- a channel changes state (a message arrived, or space freed),
- a clock reaches a time ([[time_and_clocks]]).

Everything that blocks reduces to this. A sleep or a timer is parking on a clock condition. A blocking receive is parking on a channel condition. A device interrupt is the device unparking a driver task ([[driver_model]]). There is no separate timer subsystem and no separate blocking-IPC subsystem. They are one primitive with different conditions. The adaptive default is spin-then-park: spin briefly to catch the common case with no scheduler round-trip, then park if the condition has not arrived.

Park returns a **wake reason** rather than just resuming, because the thing a task waits on can also die. A parked task must wake on more than its happy condition, or it sleeps forever on a dead peer:

- **Signaled.** The condition was met: word reached, message arrived, time came. The normal case.
- **PeerDied.** A holder of the thing being waited on is gone. This is the one liveness fact only the OS can supply, sourced from the grant arena ([[capability_lifecycle]]). The waited-on region or endpoint is an object with tracked capability-holders, and a holder's death drops its arena entry, which fires this wake to anyone parked on it. A blocking receive on a dead producer returns `PeerDied`, not an eternal sleep.
- **Revoked.** The capability backing the waited-on object was revoked (the mapped-grant teardown, [[capability_lifecycle]]). The straggler wakes here rather than faulting where it can.
- **Timeout.** A deadline on the park elapsed.

Liveness is not a separate subsystem, and it is not a header polled out of the shared page, which a hostile peer could forge. It is the arena and the scheduler meeting at the wake. The arena knows who holds a region. The scheduler delivers death as one reason among the normal ones. A spinning task that never parks reads the same fact from an OS-owned, peer-read-only status word instead: push when parked, pull when spinning.

Ownership removes accidental shared mutation, not all synchronization. Most Cathedral state stays single-owner, but shared protocol state may use atomics and library `Mutex` values under Omega's sanctioned sharing rules. A contended acquisition is written `block mutex.lock()`, or uses a suspend-based lock when the interface promises parking, so both the wait and the held-guard analysis are visible. At the scheduler boundary it still reduces to parking on a word, channel, or clock condition. Priority inheritance and wait-cycle proofs remain real concerns; ownership does not wish them away.

### Preemption, suspension, and semantic safe points

Cathedral keeps three mechanisms separate: preemption, suspension, and blocking.

| Mechanism | What happens | What it proves |
|---|---|---|
| architectural preemption | the timer saves opaque machine state and resumes the exact interrupted instruction | fairness; a compute loop cannot monopolize a core |
| suspension | an explicit `suspend` call parks the activation and later resumes it | a continuation boundary visible to source, carry checking, and lifecycle policy |
| blocking | an explicit `block` call stops the current execution thread without creating a continuation boundary | a visible wait site; bounded response only when the callee publishes a finite wait ceiling |

Every local activation receives one fixed, nonmoving stack. Omega derives its `StackPlan` from whole-call-graph WCSU, and Cathedral's start operation reserves a matching `StackLease`. Parking retains that same stack. There is no continuation-capacity negotiation and no runtime field that asserts whether continuation addresses are stable. Fixed nonmoving storage establishes that fact. Cathedral does not add a separate async language dialect or a runtime supply record for any of this.

Cathedral uses ordinary timer preemption for native and foreign execution. Preemption may occur at any instruction. It is not a source-level suspension, does not expose a half-finished state transition, and authorizes no cancellation, migration, replacement, or observation of program state. The target's checked context-switch plan is responsible for restoring exactly the interrupted state.

Every checked Omega activation uses the same canonical semantic floating-control configuration. The scheduler saves floating register contents when the selected state plan requires them, but it does not carry a per-task rounding or FTZ/DAZ mode. Foreign-call and callback trampolines are the save/restore boundary for code that may clobber those controls.

A **semantic safe point** is authored by the programmer: normally a `suspend` call, or a scheduler poll that may suspend. It is where the program exposes a lifecycle transition and where Cathedral may deliver structured cancellation, migration, or replacement. The compiler does not insert semantic safe points on loop backedges. A SIMD loop remains architecturally preemptible. If it needs bounded structured-response latency, its author chunks the work and polls between chunks.

`suspend` and `block` acknowledge calls whose statically known contract may wait. They are may-markers, not claims that the call will wait on this execution. Suspension is restricted to a direct-call or simple-let position because it creates continuation state. A blocking-only call may nest. Both markers make the important local question visible: what values, guards, and authority are held while this operation may wait?

CPU, host-thread, and address carry remain value obligations, not a runtime supply lattice. A portable activation asks nothing special. A live `SameCpu` or `SameThread` value requires Cathedral's selected start/scheduling operation to preserve that restriction, or start rejects. Cancellation is likewise an operation supplied by a cancellable runtime conformance, never a boolean in a freely authored plan. These restrictions are discharged demand by demand by the selected start/scheduling operation.

Totality rules out infinite computation. It does not by itself bound the time to the next semantic safe point. Omega's restricted fixed-work checker can close constant-bounded segments in canonical-IR fuel units. Other segments report `Unknown`, while a reached `block` or foreign edge without a finite response contract reports `NoFiniteGuarantee` and names the edge. Converting logical fuel to wall-clock time additionally depends on a derived or admitted target timing model. A future WCET analysis must re-search target paths rather than treating fuel as cycles.

### The decided mechanism: the budget-check path

There is no program-visible central budget-check trap. Checks ride existing mediation. The authority half, "no budget capability, no effect", is the reach ceiling plus an arena lookup on the `{slot, generation}` handle, already gated at the boundary crossed. The quantity half is distributed by resource kind:

- **Logical execution.** The sponsor meters canonical-IR fuel. Exhaustion is sponsor policy, not a catchable machine result.
- **Metered service quantity.** For bytes or device work (storage-write, network, NPU), the provider does an atomic decrement-if-sufficient on the caller's budget counter as it serves. The check is folded into the serve path and billed per instance. Accelerators (NPU, GPU, DPU) are metered providers on this same decrement-as-served pattern as the NIC.
- **Share.** For CPU or GPU time, the scheduler honors the weight in dispatch. There is no per-op check; the share is the enforcement.
- **Memory.** The sponsor provisions bounded `Region` or allocator capabilities, and the allocator checks them at allocation. Bump/arena residual capacity may be conserved as `CountedQuantity<Bytes>`, with a proof-level natural magnitude and `Bytes` as the quantity identity. Fragmented heaps remain fallible unless they carry placement or reservation evidence.

The scheduler is the enforcement arm, but the arm reaches into providers, dispatch, and the allocator rather than trapping centrally.

The resource vocabulary is a closed core plus driver extension. The OS understands a fixed core (cpu, memory, storage, network, gpu, npu, power), and a driver extends it by declaring a new budgetable kind. This is the same closed-core-open-extension shape as effects, failure causes, and the input registry.

A budget is a ceiling, a share, or a reservation, by resource kind. Depletable quantities get a ceiling. Time-multiplexed engines get a weighted share. Latency-guaranteed work gets a reservation. It is not one shape.

Declared intent is checked where Omega has the corresponding theorem and measured otherwise, and it remains a soft optimization. For Omega code, canonical-IR PCC can carry fixed-fuel facts. Legacy code is measured instead, and a component whose measured behavior contradicts its declared class (a `power_saving` claim that burns the battery) is demoted. PCC is not a WCET proof. A hard real-time profile needs a separate target analysis whose path search and timing model cover the whole dependency graph. The load-bearing part is the budget capability and its provider or scheduler enforcement. Intent only tunes preference within it.

Preemption needs no source cooperation. The hardware timer is always armed, and Cathedral's checked context-switch path can stop native or foreign execution at an arbitrary instruction. A forked compiler cannot remove fairness by omitting polls. Memory safety remains the non-negotiable admission condition for unwalled SAS execution, since timer preemption does not contain a forged pointer. Semantic safe points solve a different problem, structured lifecycle response. Their presence and maximum distance are checked and reported only when a contract promises such a response.

Budgets reclaim on crash by the generation bump. A budget is an arena capability. Instance death bumps the generations of its arena entries, invalidating every capability it held, budgets included, by lazy revocation. There is no leak window. Reclamation is reachability-based, the same as grants and borrows.

Global energy and thermal as a true shared budget, with arbitration under contention, belongs to [[power_management]].

### Deferred hard-control profile direction

Cathedral's general scheduler is an ordinary multicore scheduler optimized for utilization and responsiveness. A future hard-control profile is a stricter deployment selection, not Cathedral's universal execution model and not new Omega syntax.

The first credible hard-control profile should use fixed core assignment, no task migration, fixed-priority scheduling within each core, and only core-local blocking resources. Cross-core application interaction uses certified bounded channels that transfer ownership. Devices and globally mutable services belong to a designated core and are reached through those channels. The channel may use proved atomics internally, but applications do not acquire a blocking lock shared across cores. Bounded activation pools and queues make exhaustion an admission failure or a backpressure outcome, never a surprise.

Single-core priority-ceiling reasoning applies only inside one partition. It does not justify cross-core sharing. A later profile may select one proved multiprocessor resource protocol, but that selection changes scheduler semantics and blocking equations and therefore revalidates every affected deadlock and response-time certificate. The language does not name particular real-time locking protocols.

Dynamic spawning remains legal generally. Quantitative guarantees require a closed interference envelope: fixed topology, creation bounded by conserved permits, an enforced admission rate, or a proof quantified over the dynamic structure. Externally generated arrival rates are admitted facts unless an ingress limiter enforces the admitted-work rate. Receiving and rejecting excess traffic still consumes separately bounded work.

Partitioning trades opportunistic load balancing for stable locality and tight latency bounds. To avoid turning that profile into an application-architecture retrofit, Cathedral should build bounded ownership-transfer channels before it encourages shared-memory locks. The scheduler/provider remains replaceable. Changing it invalidates scheduling evidence, not ordinary source or ABI.

Core partitioning alone does not bound shared-cache, memory-bus, interconnect, or interrupt interference. A hard-control platform profile must isolate those resources or publish derived or admitted target timing bounds for them.

## Concerns & Design Space

- **Capability-gated resource access.** A held budget is required to touch a resource, not just to be prioritized within it. No budget means no effect, audited like any capability.
- **Per-component budgets & accounting.** Every instance ([[component_model]]) is a billable entity, and the OS can always answer "what is this spending."
- **Heterogeneous compute.** The CPU is one of several engines. The NPU, GPU, and DPU are schedulable, isolated, budgeted resources, multiplexed like any shared device ([[driver_model]]). An agent's inference ([[agents_as_principals]]) is scheduled and metered on the NPU the same way CPU time is on the CPU.
- **Deadline & realtime-ish scheduling.** `realtime_audio` and `interactive_ui` need bounded latency without the hard real-time guarantees most apps cannot honor.
- **Park/unpark on a condition.** The one blocking primitive: a task parks until a word, channel, or clock condition is signaled, and the signaler unparks it. Park returns a wake reason (`Signaled`, `PeerDied`, `Revoked`, `Timeout`), so a parked task wakes on the death or revocation of what it waits on, not only its happy condition. `PeerDied` and `Revoked` are sourced from the grant arena's region membership ([[capability_lifecycle]]), the one liveness fact only the OS can supply. Spin-then-park is the default policy. Pure spinning is reserved for dedicated cores and reads the same death fact from an OS-owned read-only status word ([[ipc_and_service_invocation]], [[time_and_clocks]]).
- **Side-channel isolation class.** A domain carries a declared `IsolationClass` (`baseline → flush → partition → exclusive`, [[kernel_architecture]]), and the scheduler is its enforcement arm, the same role it plays for budgets. At each context switch it reads the pair of levels and does the temporal defense: exit-flush the outgoing domain's cache footprint if it was hardened (PRIME+PROBE), and entry-flush branch predictors before an incoming hardened domain runs (Spectre-v2). `baseline↔baseline` flushes nothing. For `partition` and `exclusive` it makes the spatial placement call. The unit is the physical core, whose SMT threads share L1 and predictors, so `exclusive` gives a hardened domain the whole core and `partition` co-locates only equally-trusted work, plus L3 partitioning. The level is raised statically (a component field) or per-span (an ownership-scoped guard). Declaring above `baseline` is gated at spawn by capability, and a missing grant fails admission and never silently downgrades.
- **Priority inversion.** A high-priority component can block on a low-priority one holding a resource. Inheritance or donation must be modeled, not accidental.
- **Backpressure propagation.** When a downstream service is budget-saturated, pressure must flow upstream as a typed signal the caller can act on ([[error_model_and_recovery]], [[ipc_and_service_invocation]]).
- **Task storage is provider custody, not task-handle identity.** `Task<T>` is a linear lifecycle claim; the runtime owns execution custody. Cathedral's reference `ArenaTaskPool` provisions fixed nonmoving stacks from compiler-derived `StackPlan`s, with dynamic availability handled by `start` contracts or a fallible `try_start`. Inline completion may avoid creating a task at all, but a started local activation retains its one provisioned stack until completion. Supervisors are ordinary application data holding Task claims. They need not own stacks.
- **Memory pressure & OOM policy.** Replace the OOM killer with a negotiated, policy-driven reclaim: components declare shrinkable caches, and the OS asks before it takes.
- **Energy & thermal accounting.** Per-component energy attribution and thermal budget are governed resources, surfaced to the user ([[power_management]], [[observability_and_introspection]]).
- **Fairness across tenants.** Budgets must compose hierarchically so one tenant cannot starve another ([[component_model]] tenant axis).
- **Nested budgets.** Budgets sub-allocate down the tree. A component with a budget hands attenuated slices to its children and can never give more than it holds, so hierarchical fair-share and per-VM or per-container limits are the recursive-provider pattern ([[capability_model]]) applied to resources.
- **Abuse prevention.** A component cannot escalate its own budget. Only a broker can mint or widen one ([[capability_lifecycle]]).
- **Zero value.** A zero executor domain inherits the parent's envelope, and a zero reservation means best-effort rather than zero share, so the zero `ResourceIntent` is the inherit-or-default shape (shape 3). That choice lets a spawned task run under ambient defaults instead of failing for want of a budget ([[omega_substrate]]).

## Key Questions

- Where do deadlines come from, and who is trusted to assert `realtime_audio`? This is the flagged realtime-path revisit.
- Accelerators (NPU/GPU/DPU) are scheduled and isolated across components by the same multiplexed-queue pattern as the NIC ([[networking]], [[driver_model]]). Is preemption even possible on them?

## Omega Leverage

- Resource rights as capabilities (values plus domains), such as `Storage::Writable` with a budget domain, reuse the whole authority machinery ([Omega Chapter 19: Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- `reaches` already names the reachable resource services (`Storage`, `Network`, device providers). A budget capability is the runtime gate on each.
- A component's lifecycle `state` graph exposes natural scheduling points. The scheduler can see which state an instance is in, not just that it is runnable.
- Omega models logical execution fuel and can conserve selected quantities such as bounded arena capacity. Cathedral still owns runtime shares, service accounting, physical reservations, and deadline policy. A hard deadline becomes an authored contract only in a profile that also supplies target WCET evidence.

## Open Questions

- **Execution profile implementation.** The semantics above are fixed: fixed nonmoving WCSU-sized stacks, arbitrary timer preemption for fairness, semantic safe points for structured lifecycle actions, and `suspend` and `block` as source acknowledgements over independent operational ceilings. What remains to build is the context-switch implementation, `StackLease` provisioning, canonical-IR fuel metering, restricted fixed-work segment checking, and attributed response reporting.
- **The proof relationship.** This scheduler supplies the fairness, wake-correctness, placement, and timing evidence used by Omega's conditional progress theorems. Ownership and sanctioned access remain race-safe under an adversarial scheduler. Quantitative deadlock, starvation, memory, and response properties belong to the selected deployment composition. A future enforced real-time profile must prove its scheduler and target premises. Its initial multicore architecture is the partitioned profile above, not an unqualified lift of a single-core priority-ceiling theorem.
- Can budgets be expressed tightly enough to prove a realtime component meets its deadline, or is that always runtime best-effort?
- Is global energy/thermal a true shared budget all components draw from, and how is that arbitrated fairly under contention?

## Related
- [[capability_model]] — resource rights are capabilities in the same graph.
- [[component_model]] — the instance as the unit of budget and accounting.
- [[power_management]] — energy, thermal, and wakefulness as governed resources.
- [[observability_and_introspection]] — resource use as a queryable surface.
