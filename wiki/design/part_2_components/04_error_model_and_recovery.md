# Chapter 04: Error Model & Recovery

> Failures are typed values that carry their cause and the chain that produced them, and recovery is designed rather than improvised. This chapter owns the taxonomy of how things break and how they recover.

## The Legacy Model

Failure handling in traditional OSes is largely unprincipled. A syscall returns `EIO` and the caller has little to act on: `EIO` does not say what failed, why, whether retrying helps, or whether the state is now consistent. `errno` is a flat integer namespace shared across every cause. Signals (`SIGSEGV`, `SIGKILL`) deliver failure as an abrupt, lossy interrupt. Recovery is left to each program: a retry loop, a watchdog, a restart script, a core dump. There is no shared notion of causality. You get the symptom, never the chain that produced it, and certainly not a machine-readable one.

## The Cathedral Model

Cathedral has a typed **error taxonomy**. Failures are values carrying structured causality, and recovery is a designed behavior rather than something improvised per program. The OS knows the kinds of failure that exist, because there are finitely many that matter:

```omega
data Failure {
    cause:       FailureCause;
    component:   ComponentRef;
    causality:   CausalChain;   // what led here, not just the symptom
    recoverable: Recovery;      // retry | restart | migrate | escalate | fatal
}

data FailureCause {
    case ComponentCrash;
    case CapabilityRevoked;
    case ResourceExhausted;
    case StorageUnavailable;
    case MigrationFailed;
    case VersionMismatch;
    case ProtocolViolation;
    case DeadlineExceeded;
    case UserDeniedAuthority;
    case DeviceDisappeared;
    case NetworkPartition;
    case PowerLoss;
    case CorruptionDetected;
}
```

Every failure has four properties:

- **Typed.** A known cause, not an integer.
- **Recoverable.** A declared recovery strategy.
- **Observable.** It appears in the introspection surface ([[observability_and_introspection]]).
- **Policy-controllable.** The response is configurable, not hard-coded.

### The decided mechanism

Expected, modeled failures are `Result`-shaped values, and traps are for the residue the proof system cannot rule out. Not-found, timeout, conflict-retry, `CapabilityRevoked`, `UserDeniedAuthority`, and `DeadlineExceeded` are `Result` values (`Ok | Err`): normal control flow, matched and handled. A trap, meaning panic and then crash, is for what cannot be proved away: a hardware fault, a boundary or FFI contract violation (a provider broke a guarantee Omega accepted on trust), resource exhaustion (`ResourceExhausted`, out of memory), `CorruptionDetected`, `PowerLoss`, or a `panic` the code raises itself. A trap is never for a proven invariant. Omega proves those at compile time, so they cannot fail at runtime and there is nothing to trap. A trap means the world broke its promise or the component is out of resources. It crashes to its supervisor rather than being handled in line.

The recovery disposition is a closed set and the cause is an open one, and both are wire-encoded. The **recovery disposition** is the closed, frozen set (`retry | restart | migrate | escalate | fatal`). `FailureCause` is extensible: ordinary numbered data under a selected wire codec ([[versioned_state_and_migration]]). This is the forward-compatibility mechanism. A new OS can emit a cause an old app has never heard of, and the old app still reacts correctly because it reads the disposition even when the specific cause is unknown. The closed handling vocabulary is the semantic floor; the open causes are the detail. The closed set also carries outcome certainty. A timed-out call is `Unknown`: it may have happened, so the caller reconciles through an idempotency key rather than retrying blindly. That is distinct from a clean `Rejected`, which definitely did not happen and is safe to retry. A caller must know which before retrying ([[ipc_and_service_invocation]]).

Provenance is a non-issue for the crash path, so a component cannot lie about its own failure. The crash report is emitted by the trusted kernel, which observed the fault. A faulting component does not narrate its own death, so it cannot forge the report. A component's self-reported reason is an ordinary `Result` claim, used for diagnostics and never obeyed blindly; the retry and escalation policy belongs to the supervisor. Attestation is relative to the trust boundary: a Matrix's synthetic kernel attests within its own fiction by design ([[identity_and_principals]]).

Recovery is crash-only, then reattach, then replay. A crashed component is restarted by its supervisor from clean state ([[component_model]] supervision tree). It then reattaches to its last-committed durable state ([[memory_and_persistence]]) and replays its outbox (the output-commit pattern, [[transactions_and_consistency]]). A restart therefore resumes from the last consistent commit and re-drives any pending external effect idempotently. Crash-only composes the persistence, transaction, and outbox work. It is not new machinery.

Outcomes fail closed on zero. The success/failure discriminator must never read a zeroed or uninitialized outcome as success: a zeroed `Result` is `Err` or pending, not `Ok`. This is distinct from a zeroed `Failure` descriptor, which is the empty no-failure-described value; the discriminator is what fails closed. It is the opposite of the C `errno` hole where 0 means success.

## Concerns & Design Space

- **The taxonomy itself.** `FailureCause` is extensible rather than a closed enum the OS owns, so drivers and services add domain-specific causes ([[driver_model]]). The closed part is the disposition.
- **Causality, not symptoms.** A failure carries the chain that produced it (a revoked capability led to a denied write, which stalled a job), so postmortems are reads, not reconstructions ([[audit_compliance_provenance]]).
- **Restart as a component concept.** Restart is supervision over the component family ([[component_model]]). The crash boundary and the restarter are different components (the Erlang lesson), and state identity may survive a code restart ([[updates_and_hot_swap]]).
- **Recovery strategies.** Retry (idempotent), restart (stateless), migrate (move off a failing device or host), escalate (to a supervisor), fatal (give up loudly). Which is legal depends on the cause and the component's contract.
- **Partial failure & backpressure.** A `DeadlineExceeded` or `ResourceExhausted` downstream must surface as a typed failure upstream, feeding the scheduler's backpressure ([[scheduler_and_resources]]).
- **Failure during upgrade.** `MigrationFailed` and `VersionMismatch` are their own causes with their own rollback story ([[updates_and_hot_swap]], [[versioned_state_and_migration]]).
- **Corruption & loss.** `CorruptionDetected` and `PowerLoss` need crash-consistency guarantees to even be detectable ([[memory_and_persistence]], [[power_management]]).
- **Authority failures as normal control flow.** `CapabilityRevoked` and `UserDeniedAuthority` are ordinary outcomes, not exceptions ([[capability_model]], [[capability_lifecycle]]).
- **Zero value.** A zeroed `Failure` is the no-error case (shape 1, valid-empty). Zero must read as success with an empty causal chain, so the absence of a failure is the same value as a freshly zeroed one, and matching on a zero cause never trips a spurious error path ([[omega_substrate]]).

## Key Questions

- Who decides recovery policy: the failing component, its supervisor, or system policy ([[configuration_and_policy]])?
- How much causality can be carried cheaply enough to attach to every failure without it becoming a performance tax?

## Omega Leverage

- `Failure` is ordinary `data`, so causes, causality, and recovery are inspectable, matchable, and total, not an integer to guess at.
- Traps give the abrupt-failure path a typed home distinct from expected errors. The boundary between the two is a design decision Omega makes expressible.
- `state`/`transition` graphs model recovery directly: `failed`, `restarting`, `migrating`, and `escalated` are real states a supervisor inspects ([Omega Chapter 4: States And Transitions](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_states_transitions.md)).
- Reach and authority flow make `CapabilityRevoked` and `ResourceExhausted` predictable. The graph already knows which authorities and budgets a component depends on, so it knows what can fail.
- Omega does not define a standard causality-chain representation or a supervision model. Both are Cathedral runtime structure over typed failures.

## Open Questions

- Can causality chains be bounded in size so they are always affordable to carry?
- Is there a single recovery DSL or policy language, or is recovery ordinary component code in a supervisor?
- How do failures cross the [[distributed_boundary]] without losing their typing? Does `NetworkPartition` look the same locally and remotely?

## Related
- [[component_model]] — crash boundary, restart unit, supervision tree.
- [[versioned_state_and_migration]] — migration failure and rollback.
- [[updates_and_hot_swap]] — failure during a live upgrade.
- [[driver_model]] — device-specific failure causes and recovery.
- [[observability_and_introspection]] — failures as an observable surface.
