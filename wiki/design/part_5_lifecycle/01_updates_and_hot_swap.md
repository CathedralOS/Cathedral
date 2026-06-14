# Chapter 01: Updates & Hot Swap

> The operational act of updating a live system without stopping the world: prove replacement compatibility, reach quiescence, migrate live state, switch protocol versions, observe health, and roll back if needed.

## The Legacy Model

Updating a live system is, almost everywhere, a controlled outage: stop the thing, replace files on disk, restart, with no check that the new code accepts the old state. The dynamic linker, the service manager, and the database migration tool each own a slice of this and none of them coordinate. There is no system-wide notion of "reach a safe point, carry the live object graph forward, switch the protocol version atomically." Rollback means restoring a backup, and draining in-flight work is rarely handled at all. The restart is treated as free because the alternative was never built.

## The Cathedral Model

Update is a **first-class language/runtime/OS operation**, designed in from day one. Cathedral replaces a live component by: driving it to **quiescence**, migrating its live state forward, switching its protocol endpoints to the new version, observing the new version's health, and **rolling back** if it misbehaves — all without restarting the world.

This chapter owns the *operational* act. The typed state-shape continuity primitive — versioned `data` and the `Upgradable` migration ([Omega ch21](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)) — lives in [[versioned_state_and_migration]]; this chapter is what *drives* that machinery across a running system. This is the domain where Cathedral is genuinely differentiated, and the chapter the whole "resumability" thesis is accountable to.

### The decided mechanism

The substrate is settled; the residue is one hard corner (devices).

- **Quiescence is the actor's receive-loop park.** Run-to-completion actors ([[scheduler_and_resources]], [[component_model]]) hold no call stack between messages, so a parked actor is just its `self`. Quiescence stops being a research problem and becomes "wait for the next inter-message gap," which run-to-completion guarantees arrives promptly. The Omega swap obligations are discharged at that park — no stack to unwind, no scheduled re-entry.
- **The cutover is a pointer rebind, not a code patch.** Stop delivery (messages queue, OS-held) → run the migration (`Upgradable`, with an effectful `capture` first if old state alone is not enough, [Omega ch21](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)) → rebind the instance's one code pointer to the freshly-loaded new image → resume. The resume point is a **state tag**, version-stable, not a raw instruction pointer, so the parked task re-enters the new image by dispatch and there is no raw return address to dangle. The image loads as a second copy; the old is freed at refcount-zero (a rolling swap keeps both mapped). No in-place `.text` patching, no trampolines.
- **The cutover atom is the instance, bounded by reference edges.** A migration rewrites a data shape, so everything sharing that representation moves together: inline/embedded data migrates with it; references (handles, channels, capabilities) are the cut points. That blob — `data` plus the machines over it — is the unit. A **component** is that unit drawn as a trust+swap boundary (isolation + crash + capability); a **sub-component** is a swap-only boundary inside one trust domain (an indirect-call optimization barrier, not isolation). Both are one `boundary` parameterized by `{host, trust, swap}` — no separate `component` keyword — and where you draw the line sets swap granularity, call cost, and blast radius at once.
- **Live upgrade is single-step.** A live component is always at the last-installed version, so the upgrade is `prev → current`. The multi-version case is persisted data = `wire data` ([[ipc_and_service_invocation]]), not this. Coexistence (old + new running, versioned dispatch) is reserved for a genuinely incompatible protocol change, used sparingly.
- **The replacement is an owned, OS-gated plan.** `quiesce → capture → upgrade → install → resume`, gated on an upgrade capability ([[capability_lifecycle]]); the compiler verifies each phase's obligations chain. `capture` is the only fallible point and aborts before `old` is mutated, so a failed cutover is "did nothing."
- **Failure falls down a ladder, never force-freezes.** A `Quiesce` control message asks a component to reach a swappable rest point cooperatively; the OS never freezes a running task mid-execution. Past a deadline, the backstop is the component's declared policy ([[component_model]] `QuiescePolicy`/`UpgradePath`): **live-migrate** → **kill-and-restart** (a component that declares itself safely restartable, losing in-flight state) → **defer-to-reboot** (a critical stateful component that can neither migrate nor safely restart). "Defer to reboot" is scoped to the smallest possible set — ideally just the privileged core.

## Concerns & Design Space

- **Quiescence detection.** Proving no thread, queued transition, timer, interrupt continuation, or callback can re-enter the old code before the swap — Omega's swap safety obligations made operational.
- **Live object-graph migration.** Carrying the running state graph (including held capabilities, see [[capability_lifecycle]]) forward in one step, not just on-disk records.
- **Versioned protocol endpoints.** A component's wire surface may serve old and new protocol versions across the cutover ([[ipc_and_service_invocation]]).
- **Compatibility proofs.** Replacement compatibility — states, params, effects, exported calls — is checked before the swap is attempted, not discovered after.
- **Rolling, partial & planned upgrades.** Upgrading a subset; dependency-graph upgrade *planning* (what must move together, in what order); upgrades as transactions ([[transactions_and_consistency]]).
- **Runtime rollback & in-flight draining.** Drain outstanding requests; if the new version is unhealthy, reverse the transition — including reverse migration.
- **Leased capabilities during upgrade.** Authority a component holds must survive (or be safely re-leased across) the swap without a revocation window ([[capability_lifecycle]]).
- **Multi-version concurrency.** When quiescence is impractical, the explicit coexistence mode: versioned dispatch and old-callback fencing, used sparingly.
- **Upgrade observability.** Health, drain progress, and blocked-swap reasons are queryable ([[observability_and_introspection]]).

## Key Questions

Quiescence (the receive-loop park) and the cutover atom (the instance bounded by reference edges) are resolved above for software actors. The residue:

- **Device quiescence.** Proving a driver's *hardware* is quiescent enough to snapshot — in-flight DMA drained, interrupts masked, queue heads stable — has no run-to-completion guarantee to lean on; this is the genuinely hard corner ([[driver_model]]), and where load/swap-time checks rather than static proof likely dominate.
- **Admitting coexistence.** Single-step covers the compatible case; what does the runtime require to *safely* admit the sparingly-used old+new mode (versioned dispatch, old-callback fencing) for a protocol-incompatible change, without it becoming a permanent fork?
- **Lossy rollback.** Capture-before-mutation makes the common abort "did nothing"; but once a *forward* migration has committed and was lossy, reverse is opt-in and some upgrades are one-way — what contract tells an operator which, before they pull the trigger?

## Omega Leverage

- **Versioned `data` + migration machines** (chapter_21) supply the state-continuity substrate this chapter drives — see [../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **Quiescence / replacement obligations** — Omega already frames swap safety as borrow, invariant, effect, and scheduled-work facts; Cathedral consumes those as the precondition for a live cutover.
- **Machines as swap points** — a machine's public states and calls are its replacement contract; the OS swaps at machine boundaries.
- **`wire data`** gives versioned protocol endpoints for the cutover window.
- Omega's coexistence mode is the language hook for multi-version concurrency; the *operational policy* over it is Cathedral's to define.

## Open Questions

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
