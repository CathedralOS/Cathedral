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

### Preemption and safepoints

A task runs until it parks, reaches a selected safepoint, or is interrupted by a
runtime whose admitted contract permits asynchronous preemption. Omega does not
color machines with `async` and does not require an `await` call-site marker.
Suspension, CPU/thread affinity, and address stability are independent value
demands. Suspension is checked locally against possible `Suspend` reach; the
runtime cannot erase that ceiling. The runtime separately states preemption
granularity, migration/affinity support, and continuation-storage behavior, and
admission joins the CPU/thread/address demands against that behavior. Unknown
or unreceipted host behavior is pessimistic and therefore fails closed. Omega's
normalized admission carrier now requires the shared provider-plan receipt to
cover both the selected plan identity and the complete runtime-behavior
statement; changing pinning, capacity, preemption, cancellation, or storage
behavior is receipt drift rather than a freely asserted scheduler promise.
Cathedral's scheduler provider must supply that receipt through ordinary
provider admission before any activation may rely on the narrower behavior.

Cathedral's born-strict native profile uses boundary/back-edge safepoints and
stable continuation storage. This keeps save points enumerable and cheap while
the hardware timer requests scheduling promptly. Foreign/unproved code behind a
hardware wall uses conventional full-context timer preemption. A future
asynchronous native runtime remains legal, but must preserve complete machine
state and satisfy every live value's carry demands; it is not the default hidden
inside the language.

Totality *strengthens* this rather than replacing it. If **actor handlers are required to terminate** (the receive loop itself stays productive, [[omega_substrate]]), the *quiescence* worry dissolves by construction — a terminating handler always returns to its park, so the actor always reaches a swap point without being forced; preemption then covers only the *fairness/latency* axis (a long-but-terminating handler still shares the core). For the **realtime class**, a task that can *prove a bounded worst-case runtime* (WCET-style) may run un-preempted to a deadline — an opt-in for small RT code, bottoming out at the hardware timing model (the same below-the-model wall as constant-time). *(Exploration, not committed.)*

| Code | Mechanism | Guarantee |
|---|---|---|
| general native | boundary/back-edge safe-point scheduling | bounded reference-profile latency, enumerable live state |
| actor handlers / pure fns | required-total | no infinite loops; quiescence always reachable |
| realtime-critical | optional bounded-runtime (WCET) proof | run un-preempted to a deadline |

This is Cathedral provider policy, not a theorem that suspension and preemption
are the same axis. The normalized product model intentionally remains valid if a
later real-time provider chooses asynchronous native preemption.

### The decided mechanism: the budget-check path

**There is no central budget-check trap; the check rides existing mediation, split in two.** The **authority** half — "no budget capability → no effect" — is the **effect ceiling + an arena lookup** (the `{slot, generation}` handle), already gated at the boundary you cross; nothing new. The **quantity** half is distributed by resource kind to wherever the resource is *already* mediated:

- **Metered (bytes / cycles — storage-write, network, NPU):** the **provider** does an atomic *decrement-if-sufficient* on the caller's budget counter as it serves; folded into the serve path, billed per-instance.
- **Share (CPU / GPU time):** the **scheduler** honors the weight in dispatch — no per-op check; the share *is* the enforcement.
- **Ceiling (memory working-set):** the **allocator** checks the envelope at the alloc boundary (an `await`).

So the scheduler is the enforcement *arm*, but the arm reaches into providers, dispatch, and the allocator rather than trapping centrally.

**The resource vocabulary is a closed core + driver-extensible** — a fixed core (cpu / memory / storage / network / gpu / npu / power) the OS understands, extended by a driver declaring a new budgetable kind: the same closed-core-open-extension shape as effects, failure causes, and the input registry.

**A budget is a ceiling, a share, or a reservation — by resource kind** (ceiling for depletable quantities; weighted share for time-multiplexed engines; reservation for latency-guaranteed work). Not one shape.

**Declared intent is *proven* for Omega code, *measured* only for legacy — and is a soft optimization either way.** Cathedral-native code can *prove* its resource behavior (the WCET/bounded-runtime proof generalized — PCC), so its declared class is **trusted because proven, not measured-and-demoted**; the **measure-and-demote heuristic is the legacy-only fallback** for code that declares nothing. And honestly, intent is a *soft scheduling optimization of uncertain marginal value* (modern schedulers do well with little of it). **The load-bearing part is the budget *capability* — the hard ceiling that bounds safety and isolation; intent only tunes preference *within* it.** Lying (legacy) costs scheduling preference, never the budget.

**Preemption needs no cooperation — and the guarantee is the *checker*, not the compiler.** A producer can fork the compiler and strip the safepoints, so "the compiler inserted them" guarantees nothing — pedigree proves nothing. What guarantees it is **trust-by-checking** (the bootstrap lattice): admission to native, *unwalled* SAS execution requires the binary to **carry a proof of preemptibility** (a back-edge safepoint at every loop) that the trusted **checker** validates. Strip the safepoints → the binary **fails the check** → it is **denied native execution and runs behind the hardware-isolation wall** like any untrusted code, where the hardware timer preempts it with a full context save. So a forked compiler buys nothing: either the code carries the safepoint proof (cheap safepoint preemption, in the SAS) *or* it is walled (hardware-timer preemption). The compiler is a producer; the **checker is the gate** — and legacy/non-Omega code, carrying no such proof, simply takes the walled hardware-timer path by default.

**The timer is the liveness floor; safepoints are a *cheapness* optimization, not a liveness or safety requirement.** Split the SAS check in two. **Memory-safety** is the non-negotiable one — it is what isolates SAS components at all, and (via ownership / no-data-races) what makes preempting a task at *any* point *safe*: preempt anywhere, run another, and nothing corrupts because no task can touch another's owned data. **Safepoints** only make preemption *cheap* (a bounded continuation vs a full register save). So weakening the *safepoint* check **cannot hang the box**: the hardware preemption timer is **always armed** and fires regardless of address space, so a spinner — or a task that lied about being cooperative — is forced off with a bounded full-context save, *and* the scheduler reads "needed a timer-kick instead of yielding" as a behavior signal and **de-ranks it hard**. Bad behavior is self-penalizing (slow preemption + demotion), so **PCC-for-safepoints is a performance optimization, not a correctness requirement** — the incentive replaces the proof on the cooperation axis. The one line that stays bright: this holds for the **liveness/fairness** axis *only*. **Memory-safety is not self-penalizing** — a forged pointer reads your data, it does not merely run slow — so it is the single check that can never be relaxed, covered by the MMU between apps and by proof-or-CHERI inside the SAS ([[kernel_architecture]]).

**Budgets reclaim on crash by the generation bump.** A budget is an arena capability; instance death **bumps the generations** of its arena entries, invalidating every capability it held (budgets included) by lazy revocation — no leak window, reachability-based, the same as grants and borrows.

*(Global energy/thermal as a true shared budget — arbitration under contention — is deferred to [[power_management]].)*

## Concerns & Design Space

- **Capability-gated resource access.** A held budget is required to *touch* a resource, not just to be prioritized within it. Absence of budget = absence of the effect, audited like any capability.
- **Per-component budgets & accounting.** Every instance ([[component_model]]) is a billable entity; the OS can always answer "what is this spending."
- **Heterogeneous compute.** The CPU is one of several engines. The NPU, GPU, and DPU are first-class schedulable, isolated, budgeted resources, multiplexed like any shared device ([[driver_model]]). An agent's inference ([[agents_as_principals]]) is scheduled and metered on the NPU exactly like CPU time on the CPU.
- **Deadline & realtime-ish scheduling.** `realtime_audio` and `interactive_ui` need bounded latency without true hard-RT guarantees most apps can't honor.
- **Park/unpark on a condition.** The one blocking primitive: a task parks until a word, channel, or clock condition is signaled, and the signaler unparks it. Park returns a **wake reason** (`Signaled` / `PeerDied` / `Revoked` / `Timeout`), so a parked task wakes on the death or revocation of what it waits on, not only its happy condition — `PeerDied` and `Revoked` are sourced from the grant arena's region membership ([[capability_lifecycle]]), the one liveness fact only the OS can supply. Spin-then-park is the default policy; pure spinning is reserved for dedicated cores and reads the same death fact from an OS-owned read-only status word ([[ipc_and_service_invocation]], [[time_and_clocks]]).
- **Side-channel isolation class.** A domain carries a declared `IsolationClass` (`baseline → flush → partition → exclusive`, [[kernel_architecture]]), and the scheduler is its enforcement arm — the same role it plays for budgets. At each context switch it reads the *pair* of levels and does the **temporal** defense: exit-flush the outgoing domain's cache footprint if it was hardened (PRIME+PROBE), entry-flush branch predictors before an incoming hardened domain runs (Spectre-v2); `baseline↔baseline` flushes nothing. For `partition`/`exclusive` it makes the **spatial** placement call — the unit is the physical core, whose SMT threads share L1 + predictors, so `exclusive` gives a hardened domain the whole core and `partition` co-locates only equally-trusted work (plus L3 partitioning). The level is raised statically (a component field) or per-span (an ownership-scoped guard); declaring above `baseline` is gated at spawn by capability, and a missing grant **fails admission, never silently downgrades**.
- **Priority inversion.** A high-priority component blocked on a low-priority one holding a resource — inheritance/donation must be modeled, not accidental.
- **Backpressure propagation.** When a downstream service is budget-saturated, pressure must flow upstream as a typed signal the caller can act on ([[error_model_and_recovery]], [[ipc_and_service_invocation]]).
- **Task storage is provider custody, not task-handle identity.** `Task<T>` is a linear lifecycle claim; the runtime owns execution custody, and physical activation storage may belong to an Arena-backed pool, the hosted OS, a remote provider, or nowhere after inline completion. Cathedral's reference `ArenaTaskPool` provisions bounded stable slots from compiler-derived activation/WCSU demand, with dynamic availability handled by `start` contracts or a fallible `try_start`. That map is the bounded Cathedral profile, not universal Omega semantics. Supervisors are ordinary application data holding Task claims; they need not own stacks.
- **Memory pressure & OOM policy.** Replace the OOM killer with a negotiated, policy-driven reclaim: components declare shrinkable caches, the OS asks before it takes.
- **Energy & thermal accounting.** Per-component energy attribution and thermal budget as first-class, surfaced to the user ([[power_management]], [[observability_and_introspection]]).
- **Fairness across tenants.** Budgets must compose hierarchically so one tenant cannot starve another ([[component_model]] tenant axis).
- **Nested budgets.** Budgets sub-allocate down the tree: a component with a budget hands attenuated slices to its children and can never give more than it holds, so hierarchical fair-share and per-VM or per-container limits are the recursive-provider pattern ([[capability_model]]) applied to resources.
- **Abuse prevention.** A component cannot escalate its own budget; only a broker can mint or widen one ([[capability_lifecycle]]).
- **Zero value.** A zero executor domain inherits the parent's envelope and a zero reservation means best-effort rather than zero share, so the zero `ResourceIntent` is the inherit-or-default shape (shape 3): a deliberate choice that lets a spawned task run under ambient defaults instead of failing for want of an explicit budget ([[omega_substrate]]).

## Key Questions

*(Mostly resolved by "The decided mechanism: the budget-check path": vocabulary = closed core + driver-extensible; a budget is ceiling/share/reservation by resource kind; intent is *proven* for Omega (PCC), *measured-and-demoted* only for legacy, and a soft optimization regardless — the budget *capability* is the load-bearing part; accelerators (NPU/GPU/DPU) are metered providers, the same decrement-as-served pattern as the NIC. Deadlines/realtime trust stays with the flagged realtime-path REVISIT.)*

- What is the canonical resource vocabulary, and is it open (drivers add new resource kinds) or fixed?
- Is a budget a hard ceiling, a weighted share, a reservation, or all three depending on resource kind?
- How does declared intent reconcile with measured behavior — does the OS demote a component that lies about being `power_saving`?
- Where do deadlines come from, and who is trusted to assert `realtime_audio`?
- How are accelerators (NPU/GPU/DPU) scheduled and isolated across components: the same multiplexed-queue pattern as the NIC ([[networking]], [[driver_model]]), and is preemption even possible on them?

## Omega Leverage

- Resource rights as **capabilities (values + domains)** — `Storage::Writable` with a budget domain — reuse the whole authority machinery ([../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- **`effects`** already name the resource-touching boundaries (`filesystem_io`, `device_io`, `network_io`); a budget capability is the runtime gate on each.
- A component's lifecycle **`state` graph** exposes natural scheduling points — the scheduler can see *which state* an instance is in, not just that it's runnable.
- Omega does **not** model resource *quantities* or deadlines as proof obligations; budget arithmetic, accounting, and enforcement are Cathedral runtime — a candidate for bounded-value contracts to grow into.

## Open Questions

- **Preemption profile — direction resolved, implementation pending.** Cathedral starts with safe-point scheduling and stable continuation storage because it is the strict, bounded profile. Omega checks suspension locally against effects and keeps CPU migration, host-thread migration, address stability, and preemption granularity independent in normalized runtime contracts. An asynchronous provider is admissible when its full-context machinery and behavior satisfy every relevant live value's demands; hard-real-time evidence may force that provider later. The bootstrap scheduler remains an explicitly bounded static subset, but this is a provider/package restriction rather than a `spawn`/`await` language dialect.
- **The proof relationship.** This scheduler is the *trusted* provider of the fairness / atomicity / wake-correctness hypotheses that discharge Omega's *conditional* liveness theorems (progress, no starvation). Omega's *safety* theorems (data-race / deadlock / protocol freedom) hold regardless of it. So a bug here cannot break a safety proof, but it can invalidate a liveness/progress guarantee — and "Cathedral provides the scheduler" *relocates* that obligation onto this chapter, it does not discharge it. If Cathedral ever promises *enforced* real-time deadlines (vs. the `realtime_audio` best-effort above), this scheduler becomes a Ravenscar-class verified scheduler whose fairness/timing is itself proven.
- Can budgets be expressed tightly enough to *prove* a realtime component meets its deadline, or is that always runtime best-effort?
- How are budgets reclaimed on crash or revocation without a window where a resource leaks?
- Is global energy/thermal a true shared budget all components draw from, and how is that arbitrated fairly under contention?

## Related
- [[capability_model]] — resource rights are capabilities in the same graph.
- [[component_model]] — the instance as the unit of budget and accounting.
- [[power_management]] — energy, thermal, and wakefulness as governed resources.
- [[observability_and_introspection]] — resource use as a queryable surface.
