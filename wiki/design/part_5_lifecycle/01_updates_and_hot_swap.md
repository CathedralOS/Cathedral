# Chapter 01: Updates & Hot Swap

> The operational act of updating a live system without stopping the world: prove replacement compatibility, reach quiescence, migrate live state, switch protocol versions, observe health, and roll back if needed.

## The Legacy Model

Updating a live system is, almost everywhere, a controlled outage: stop the thing, replace files on disk, restart, with no check that the new code accepts the old state. The dynamic linker, the service manager, and the database migration tool each own a slice of this and none of them coordinate. There is no system-wide notion of "reach a safe point, carry the live object graph forward, switch the protocol version atomically." Rollback means restoring a backup, and draining in-flight work is rarely handled at all. The restart is treated as free because the alternative was never built.

## The Cathedral Model

Update is a **first-class Cathedral runtime/OS operation**, designed in from day one. Cathedral replaces a live component by: driving it to **quiescence**, migrating its live state forward, switching its protocol endpoints to the new version, observing the new version's health, and **rolling back** if it misbehaves — all without restarting the world. Omega supplies the checked building blocks; whether the reusable phase protocol ultimately belongs in Cathedral, a target-neutral library Cathedral consumes, or a smaller language/runtime substrate remains deliberately consumer-driven rather than pre-decided.

This chapter owns the *operational* act. Explicit old/new `data` shapes and
ordinary checked migration machines ([Omega ch22](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md))
live in [[versioned_state_and_migration]]; this chapter drives them across a
running system. This is the domain where Cathedral is genuinely differentiated,
and the chapter the whole "resumability" thesis is accountable to.

### The decided mechanism

The substrate is settled; the residue is one hard corner (devices).

- **Quiescence is the actor's receive-loop park.** Run-to-completion actors ([[scheduler_and_resources]], [[component_model]]) hold no call stack between messages, so a parked actor is just its `self`. Quiescence stops being a research problem and becomes "wait for the next inter-message gap," which run-to-completion guarantees arrives promptly. The Omega swap obligations are discharged at that park — no stack to unwind, no scheduled re-entry.
- **The cutover is a pointer rebind, not a code patch.** Stop delivery
  (messages queue, OS-held) → run an ordinary checked migration machine, with
  effectful capture first when needed → atomically rebind the instance's code
  pointer to the admitted new image → resume. The resume point is a stable
  state tag, not a raw instruction pointer. The image loads as a second copy;
  the old is freed when its liveness pins reach zero. No in-place `.text`
  patching and no language-level replacement syntax.
- **The cutover atom is a swap unit bounded by reference edges.** A migration rewrites a data shape, so everything sharing that representation moves together: inline/embedded data migrates with it; references (handles, channels, capabilities) are the cut points. That blob — `data` plus the machines over it — is the unit, and in Omega the **machine** is exactly the swap point. Three axes stay separate rather than fusing into one keyword: the **package** (Omega's deployment/compile unit, [ch15](../../../../Omega/wiki/language_guide/chapter_15_modules_imports_visibility.md)) is what ships — *what Cathedral calls a component* — and one package holds many machine-granular swap points (deployment unit ≠ swap unit); **trust/isolation** is the OS's separate call (Omega-proved safety vs a hardware wall, [[kernel_architecture]]); and Omega's `boundary` ([ch19](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)) is the narrower host/FFI edge, not a swap construct. Drawing the package line sets deployment and blast radius; drawing the machine line sets swap granularity and call cost.
- **A live cutover names one explicit old/new artifact edge.** That does not
  impose a universal single-step history on persisted data or protocols.
  Bounded old/new provider coexistence is a separate runtime policy.
- **The replacement is an owned, OS-gated protocol.** `admit → quiesce →
  capture/prepare → commit → resume/retire`, gated on an upgrade capability
  ([[capability_lifecycle]]). Before commit, every failure must return a still
  meaningful old-state token; commit is the explicit point of no return and
  installs atomically. Ordinary machine contracts and linear phase values—not
  a compiler `replace` block—verify the chain.
- **Failure falls down a ladder, never force-freezes.** A `Quiesce` control message asks a component to reach a swappable rest point cooperatively; the OS never freezes a running task mid-execution. Past a deadline, the backstop is the component's declared policy ([[component_model]] `QuiescePolicy`/`UpgradePath`): **live-migrate** → **kill-and-restart** (a component that declares itself safely restartable, losing in-flight state) → **defer-to-reboot** (a critical stateful component that can neither migrate nor safely restart). "Defer to reboot" is scoped to the smallest possible set — ideally just the privileged core.

## Concerns & Design Space

- **Quiescence detection.** Proving no thread, queued transition, timer, interrupt continuation, or callback can re-enter the old code before the swap — Omega's swap safety obligations made operational.
- **Live object-graph migration.** Carrying the running state graph (including held capabilities, see [[capability_lifecycle]]) forward in one step, not just on-disk records.
- **Versioned protocol endpoints.** A component's wire surface may serve old and new protocol versions across the cutover ([[ipc_and_service_invocation]]).
- **Compatibility proofs.** Replacement compatibility — states, params, effects, exported calls — is checked before the swap is attempted, not discovered after.
- **Rolling, partial & planned upgrades.** Upgrading a subset; dependency-graph upgrade *planning* (what must move together, in what order); upgrades as transactions ([[transactions_and_consistency]]).
- **Runtime rollback & in-flight draining.** Drain outstanding requests; if the new version is unhealthy, reverse the transition — including reverse migration.
- **Leased capabilities during upgrade.** Authority a component holds must survive (or be safely re-leased across) the swap without a revocation window ([[capability_lifecycle]]).
- **Multi-provider coexistence.** When quiescence is impractical, bounded old
  and new artifacts coexist behind contract-pinned dispatch and callback
  fencing.
- **Upgrade observability.** Health, drain progress, and blocked-swap reasons are queryable ([[observability_and_introspection]]).

## Key Questions

Quiescence (the receive-loop park) and the cutover atom (the instance bounded by reference edges) are resolved above for software actors. The residue:

- **Device quiescence.** Proving a driver's *hardware* is quiescent enough to snapshot — in-flight DMA drained, interrupts masked, queue heads stable — has no run-to-completion guarantee to lean on; this is the genuinely hard corner ([[driver_model]]), and where load/swap-time checks rather than static proof likely dominate.
- **Admitting coexistence.** What must the runtime require to safely admit
  bounded old/new providers behind pinned contracts without creating a
  permanent fork?
- **Lossy rollback.** Capture-before-mutation makes the common abort "did nothing"; but once a *forward* migration has committed and was lossy, reverse is opt-in and some upgrades are one-way — what contract tells an operator which, before they pull the trigger?

## Omega Leverage

- **Ordinary data, sums, domains, and migration machines** supply the
  state-continuity substrate this chapter drives — see
  [Omega chapter 22](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md).
- **Quiescence / replacement obligations** — Omega already frames swap safety as borrow, invariant, effect, and scheduled-work facts; Cathedral consumes those as the precondition for a live cutover.
- **Machines as swap points** — a machine's public states and calls are its replacement contract; the OS swaps at machine boundaries.
- **Numbered protocol schemas and codecs** give compatible endpoints for the
  cutover window.
- Component artifacts, pinned import slots, and provider admission are Omega
  runtime/toolchain substrate; coexistence policy is Cathedral's.

## Open Questions

- **Where does the replacement framework live?** Cathedral is the only planned
  consumer, so prototype the `quiesce -> capture -> prepare/upgrade -> commit ->
  resume` protocol here rather than pre-emptively making `replace` an Omega
  language construct. The prototype must make the point of no return explicit:
  before commit, every failure leaves the old state resumable; commit either
  installs atomically or returns the still-live phase tokens unchanged. Real
  implementation pressure then decides whether the reusable protocol remains
  Cathedral code, is extracted into a target-neutral Omega library, or exposes
  an irreducible primitive that belongs in the language/runtime core. Verbosity
  alone is not evidence for new language syntax.
- Can an outstanding *borrow* of capability or state legitimately block a swap indefinitely, and is that acceptable back-pressure or a liveness bug?
- How much swap safety is statically provable vs. necessarily a load/swap-time runtime check, and who is accountable when the runtime check fails mid-upgrade?
- How does live migration interact with persistence and crash recovery ([[memory_and_persistence]]) if the system dies *during* a cutover?

## Related
- [[versioned_state_and_migration]] — the typed state-shape continuity primitive.
- [[component_model]] — what gets swapped, and its replacement contract.
- [[memory_and_persistence]] — surviving a crash mid-upgrade.
- [[package_system]] — the package that delivers a new version.
- [[driver_model]] — restartable, upgradable drivers as a hard case.
- [[capability_lifecycle]] — leased capabilities crossing a migration.
