# Chapter 04: Error Model & Recovery

> Failure is a first-class part of the system. This chapter owns the typed taxonomy of how things break and the principled way they recover.

## The Legacy Model

Failure handling in traditional OSes is largely unprincipled. A syscall returns `EIO` and the caller has little to act on: `EIO` does not say what failed, why, whether retrying helps, or whether the state is now consistent. `errno` is a flat integer namespace shared across every cause. Signals (`SIGSEGV`, `SIGKILL`) deliver failure as an abrupt, lossy interrupt. Recovery is left to each program: a retry loop, a watchdog, a restart script, a core dump. There is no shared notion of *causality* — you get the symptom, never the chain that produced it, and certainly not a machine-readable one.

## The Cathedral Model

A real, **typed error taxonomy** where failures are values carrying structured causality, and recovery is a designed behavior rather than improvised per program. The OS knows the kinds of failure that exist, because there are finitely many that matter:

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

Four properties every failure must have: **typed** (a known cause, not an integer), **recoverable** (a declared recovery strategy), **observable** (it appears in the introspection surface, [[observability_and_introspection]]), and **policy-controllable** (the response is configurable, not hard-coded).

## Concerns & Design Space

- **The taxonomy itself.** Is `FailureCause` a closed enum the OS owns, or extensible so drivers and services add domain-specific causes ([[driver_model]])?
- **Causality, not symptoms.** A failure carries the chain that produced it (a revoked capability → a denied write → a stalled job), so postmortems are reads, not reconstructions ([[audit_compliance_provenance]]).
- **Restart as a component concept.** Restart is supervision over the component family ([[component_model]]): the crash boundary and the restarter are different components (the Erlang lesson), and state identity may survive a code restart ([[updates_and_hot_swap]]).
- **Recovery strategies.** retry (idempotent), restart (stateless), migrate (move off a failing device/host), escalate (to a supervisor), fatal (give up loudly). Which is legal depends on the cause and the component's contract.
- **Partial failure & backpressure.** A `DeadlineExceeded` or `ResourceExhausted` downstream must surface as a typed failure upstream, feeding the scheduler's backpressure ([[scheduler_and_resources]]).
- **Failure during upgrade.** `MigrationFailed` and `VersionMismatch` are their own causes with their own rollback story ([[updates_and_hot_swap]], [[versioned_state_and_migration]]).
- **Corruption & loss.** `CorruptionDetected` and `PowerLoss` need crash- consistency guarantees to even be detectable ([[memory_and_persistence]], [[power_management]]).
- **Authority failures as first-class.** `CapabilityRevoked` and `UserDeniedAuthority` are normal control flow, not exceptions ([[capability_model]], [[capability_lifecycle]]).
- **Zero value.** A zeroed `Failure` is the no-error case (shape 1, valid-empty): zero must read as success with an empty causal chain so the absence of a failure is the same value as a freshly zeroed one, and matching on a zero cause never trips a spurious error path ([[omega_substrate]]).

## Key Questions

- Is the failure taxonomy closed (the OS owns it, exhaustively) or open (extensible per domain)? This shapes how much the kernel must know.
- Are errors propagated as Omega typed return values, as traps, or both — and where is the line?
- Who decides recovery policy: the failing component, its supervisor, or system policy ([[configuration_and_policy]])?
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
- How do failures cross the [[distributed_boundary]] without losing their typing — does `NetworkPartition` look the same locally and remotely?

## Related
- [[component_model]] — crash boundary, restart unit, supervision tree.
- [[versioned_state_and_migration]] — migration failure and rollback.
- [[updates_and_hot_swap]] — failure during a live upgrade.
- [[driver_model]] — device-specific failure causes and recovery.
- [[observability_and_introspection]] — failures as an observable surface.
