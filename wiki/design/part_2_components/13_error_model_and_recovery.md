# Chapter 13: Error Model & Recovery

> Failure is a first-class part of the system, not an afterthought. This chapter owns the typed taxonomy of how things break and the principled way they recover.

## The Legacy Contract

Operating systems are full of unprincipled failure. A syscall returns `EIO` — and you pray, because `EIO` tells you almost nothing: not what failed, not why, not whether retrying helps, not whether the state is now consistent. `errno` is a flat integer namespace shared across every cause. Signals (`SIGSEGV`, `SIGKILL`) deliver failure as an abrupt, lossy interrupt. Recovery is whatever each program improvised: a retry loop, a watchdog, a restart script, a core dump nobody reads. There is no shared notion of *causality* — you get the symptom, never the chain that produced it, and certainly not a machine-readable one.

## What Cathedral Wants

A real, **typed error taxonomy** where failures are values carrying structured causality, and recovery is a designed behavior — not a prayer. The OS knows the kinds of failure that exist, because there are finitely many that matter:

```omega
data Failure {
    cause:       FailureCause;
    component:   ComponentRef;
    causality:   CausalChain;   // what led here, not just the symptom
    recoverable: Recovery;      // retry | restart | migrate | escalate | fatal
}

domain FailureCause {
    ComponentCrash | CapabilityRevoked | ResourceExhausted |
    StorageUnavailable | MigrationFailed | VersionMismatch |
    ProtocolViolation | DeadlineExceeded | UserDeniedAuthority |
    DeviceDisappeared | NetworkPartition | PowerLoss | CorruptionDetected
}
```

Four properties every failure must have: **typed** (a known cause, not an integer), **recoverable** (a declared recovery strategy), **observable** (it appears in the introspection surface, [[33_observability_and_introspection]]), and **policy-controllable** (the response is configurable, not hard-coded).

## Concerns & Design Space

- **The taxonomy itself.** Is `FailureCause` a closed enum the OS owns, or extensible so drivers and services add domain-specific causes ([[24_driver_model]])?
- **Causality, not symptoms.** A failure carries the chain that produced it (a revoked capability → a denied write → a stalled job), so postmortems are reads, not reconstructions ([[34_audit_compliance_provenance]]).
- **Restart as a component concept.** Restart is supervision over the component family ([[09_component_model]]): the crash boundary and the restarter are different components (the Erlang lesson), and state identity may survive a code restart ([[23_updates_and_hot_swap]]).
- **Recovery strategies.** retry (idempotent), restart (stateless), migrate (move off a failing device/host), escalate (to a supervisor), fatal (give up loudly). Which is legal depends on the cause and the component's contract.
- **Partial failure & backpressure.** A `DeadlineExceeded` or `ResourceExhausted` downstream must surface as a typed failure upstream, feeding the scheduler's backpressure ([[10_scheduler_and_resources]]).
- **Failure during upgrade.** `MigrationFailed` and `VersionMismatch` are their own causes with their own rollback story ([[23_updates_and_hot_swap]], [[21_versioned_state_and_migration]]).
- **Corruption & loss.** `CorruptionDetected` and `PowerLoss` need crash- consistency guarantees to even be detectable ([[11_memory_and_persistence]], [[14_power_management]]).
- **Authority failures as first-class.** `CapabilityRevoked` and `UserDeniedAuthority` are normal control flow, not exceptions ([[03_capability_model]], [[04_capability_lifecycle]]).

## Key Questions

- Is the failure taxonomy closed (the OS owns it, exhaustively) or open (extensible per domain)? This shapes how much the kernel must know.
- Are errors propagated as Omega typed return values, as traps, or both — and where is the line?
- Who decides recovery policy: the failing component, its supervisor, or system policy ([[20_configuration_and_policy]])?
- How much causality can be carried cheaply enough to attach to *every* failure without it becoming a performance tax?

## Omega Leverage

- **Errors as typed values** — `Failure` is ordinary `data`, so causes, causality, and recovery are inspectable, matchable, and total, not an integer to guess at.
- **Traps** give the abrupt-failure path a typed home distinct from expected errors; the boundary between the two is a design decision Omega makes expressible.
- **`state`/`transition` graphs** model recovery directly — `failed`, `restarting`, `migrating`, `escalated` are real states a supervisor inspects ([../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)).
- **Effects + authority flow** make `CapabilityRevoked` and `ResourceExhausted` predictable: the graph already knows which authorities and budgets a component depends on, so it knows what can fail.
- Omega does **not** define a standard causality-chain representation or a supervision model — both are Cathedral runtime structure over typed failures.

## Open Questions

- Can causality chains be bounded in size so they're always affordable to carry?
- Is there a single recovery DSL/policy language, or is recovery just ordinary component code in a supervisor?
- How do failures cross the [[17_distributed_boundary]] without losing their typing — does `NetworkPartition` look the same locally and remotely?

## Related
- [[09_component_model]] — crash boundary, restart unit, supervision tree.
- [[21_versioned_state_and_migration]] — migration failure and rollback.
- [[23_updates_and_hot_swap]] — failure during a live upgrade.
- [[24_driver_model]] — device-specific failure causes and recovery.
- [[33_observability_and_introspection]] — failures as an observable surface.
