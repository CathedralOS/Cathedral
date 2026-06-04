# Chapter 01: Updates & Hot Swap

> The operational act of updating a live system without stopping the world: prove replacement compatibility, reach quiescence, migrate live state, switch protocol versions, observe health, and roll back if needed.

## The Legacy Model

Updating a live system is, almost everywhere, a controlled outage: stop the thing, replace files on disk, restart, with no check that the new code accepts the old state. The dynamic linker, the service manager, and the database migration tool each own a slice of this and none of them coordinate. There is no system-wide notion of "reach a safe point, carry the live object graph forward, switch the protocol version atomically." Rollback means restoring a backup, and draining in-flight work is rarely handled at all. The restart is treated as free because the alternative was never built.

## The Cathedral Model

Update is a **first-class language/runtime/OS operation**, designed in from day one. Cathedral replaces a live component by: driving it to **quiescence**, migrating its live state forward, switching its protocol endpoints to the new version, observing the new version's health, and **rolling back** if it misbehaves — all without restarting the world.

This chapter owns the *operational* act. The typed state-shape continuity primitive — versioned `data` and migration machines — lives in [[versioned_state_and_migration]]; this chapter is what *drives* that machinery across a running system. This is the domain where Cathedral is genuinely differentiated, and the chapter the whole "resumability" thesis is accountable to.

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

- How is quiescence proven in the presence of interrupts, timers, async work, and external hardware ([[driver_model]]) — statically, or via load-time checks?
- What is the cutover atom: per-component, per-dependency-cluster, or whole-graph?
- When is coexistence (old + new running together) worth its cost, and what does the runtime require to admit it safely?
- What is the rollback contract when forward migration was *lossy* — is reverse migration always available, or do some upgrades become one-way?

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
