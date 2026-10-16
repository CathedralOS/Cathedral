# Chapter 01: Transactions & Consistency

> Atomicity is an OS primitive, not a database feature. This chapter owns the one common mechanism by which a set of state transitions commits all-or-nothing.

## Design status

The [storage foundation proposal](../../proposals/0002_dbfs_storage_foundation.md)
records the current candidate architecture. Durable storage transactions belong
in the platform; atomicity spanning kernel authority and service state is still
an [owner question](../../../OWNER_QUESTIONS.md#kernel-mechanisms-versus-platform-orchestration).
The scope below is a design ambition, not a claim that one implemented primitive
already makes every subsystem transactional. Engine, isolation and recovery
contracts remain to be selected and tested.

## The Legacy Model

In a traditional OS, transactions live inside a database and nowhere else. Installing a package, applying an update, editing a config file, changing a permission, or moving files are all multi-step mutations with no atomicity. They half-complete and leave a state no design intended. The canonical recovery is "delete the lockfile and run it again." Each subsystem reinvents a private, partial commit/rollback (dpkg's `--configure -a`, a `.bak` file, a journal replay), and none of them compose. There is no OS-level answer to "make these N changes atomically, or none of them."

## The Cathedral Model

The intended direction is a common transactional interface that subsystems can use to make a set of state transitions atomic, isolated, and rollback-able. It is exposed the same way for filesystem writes, package installs, upgrades, UI state, configuration changes, and capability grants and revocations. The whole OS is not globally transactional; that way lies a planet-sized lock. The work is to identify which state transitions must be atomic, draw that boundary, and give it one shared commit/abort/compensation machinery. Outside those boundaries the system stays loosely coupled and eventually consistent on purpose.

A transaction reads as a scope over participating state with a proven outcome:

```omega
transaction grant_and_record {
    participates cap_table, audit_log
    requires exclusive(cap_table)
    ensures  committed | rolled_back
}
```

### Explicit commit and established database mechanisms

Durability requires an actual commit through the storage boundary. A RAM mutation,
a language transition or a function's return does not by itself persist anything.
A future compiler/runtime integration may lower an explicit durable operation to
that commit, but API spelling and transaction-overlay behavior remain proposals.
There is no implicit RAII durability guarantee.

One metadata engine can publish changes to several objects atomically. Realm
boundaries scope authority and naming, not necessarily engine ownership. Updating
two authorized realms in the same engine need not require two-phase commit.
Grouping state in one aggregate can simplify modeling, but single-object atomicity
still depends on the engine's commit/recovery protocol; it is not free.

Two-phase commit is a candidate for independently committing participants, local
or remote. It requires durable decisions, participant recovery and a policy for
in-doubt work; being on one machine does not remove coordinator failure. Sagas,
compensation and transactional outboxes serve different requirements and are not
interchangeable with atomic commit. No production cross-participant mechanism is
selected. The existing kernel/platform boundary question remains open.

Immutable chunks plus transactional metadata require explicit integration:
backing must be recoverably durable before committed metadata can reference it,
and reclamation must preserve all current, retained and actively read versions.
A log append or root update is the publication point only under a specified
ordering, integrity and recovery protocol. Flush count depends on that protocol
and device assumptions, not on object or realm count alone.

### Isolation, authority and observation

Snapshot reads and optimistic conflict detection are candidate concurrency rules.
Copy-on-write alone is not MVCC or snapshot isolation; versions need visibility,
conflict and reader-lifetime rules. Write-write conflict checks alone can permit
write skew between disjoint writes based on shared predicates. The store must
choose which cross-record invariants it promises and how read/predicate conflicts
are handled before exposing a general transaction API. This is recorded in
[owner questions](../../../OWNER_QUESTIONS.md).

A transaction must validate its required authority at a defined point relative
to publication. The exact relationship to concurrent revocation, already accepted
operations and kernel capability state remains unresolved. The Rust catalog's
connection-bound staging and root-revision checks are lab behavior, not the final
persistent authority protocol.

A candidate durable watcher records changes in the same transaction as the state
and resumes from a committed sequence cursor. This does not require a CRDT or
exposing the physical WAL. Replay needs explicit retention, authorization filtering,
expiry/resynchronization and duplicate-delivery rules. CRDTs concern convergence
of independently writable replicas, a separate decision with type-specific merge
semantics. Neither a change log nor a content hash supplies those semantics.

Irreversible effects happen outside the storage transaction. A transactional
outbox records an intent with state, then emits after commit using an idempotency
key and reconciliation where needed. It does not guarantee exactly-once external
effects. A proposed Omega reach restriction could keep such effects out of commit
code; that compiler boundary is not implemented by the Rust storage experiment.

## Concerns & Design Space

- **Where the boundaries are.** The core design work is classifying state transitions into "must be atomic together" sets. Package install plus capability grant is one transaction ([[package_system]], [[capability_model]]); a background telemetry write is not.
- **Multi-object commit.** The filesystem-as-database ([[filesystem_as_database]]) is the largest consumer: rename, content, and metadata commit as one log entry.
- **Atomic grant/revoke.** A capability grant and its authority-graph edge, or a revoke and its sub-tree invalidation, must land together ([[capability_model]]).
- **Upgrade transactions.** An upgrade is a transaction whose participants include live state migration ([[updates_and_hot_swap]]). Abort means rolling back to the prior version cleanly.
- **Rollback & compensation.** Some effects reverse (state writes). Some do not (a sent packet, a fired actuator) and need compensation, not naive undo.
- **Conflict detection & isolation.** Concurrent transactions on the same state can be handled optimistically (detect-and-retry) or pessimistically (lock-and-block), and each subsystem needs a particular isolation level.
- **Distributed transactions / sagas.** Across the distributed boundary ([[filesystem_as_database]] sync, multiple devices), two-phase commit is often wrong; long-lived sagas with compensation usually fit better.
- **Zero value.** A zero transaction has no participants and commits as a no-op (valid-empty). It satisfies `committed` trivially and leaves state untouched, so an empty commit is coherent rather than an error ([[omega_substrate]]).

## Key Questions

- What is the smallest transactional API that serves filesystem, package, config, and capability subsystems without forcing them into one storage engine?
- Which transitions are required atomic versus merely convenient atomic, and who is authorized to declare a new transactional boundary?

## Omega Leverage

- **Machines, states & transitions** model a transaction as a state graph (`open -> staged -> committed | aborted`) the compiler can inspect. See Omega [States And Transitions](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_states_transitions.md).
- **Reach** distinguishes which world-touching services may participate. A reach ceiling lets the compiler exclude operations that require compensation rather than rollback. See Omega [Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Domains** name the commit lifecycle (`Txn::Staged`, `Txn::Committed`) and let `requires`/`ensures` enforce that participants reach a terminal domain. See Omega [Domains](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_8_domains.md).
- **Ownership / borrowing** supplies the isolation substrate. Exclusive access is already a proven fact, so pessimistic isolation can lean on the borrow checker.
- Omega does not model distributed commit or saga coordination; that is OS runtime policy layered over the typed local primitive.

## Open Questions

- Can the borrow checker alone provide serializable isolation for in-memory participants, or is a runtime conflict detector always required?
- Is there one transaction manager, or per-domain managers that federate? How is a cross-subsystem transaction coordinated without a global bottleneck?
- How long may a transaction hold participants before it blocks hot swap or starves other writers ([[updates_and_hot_swap]])?

## Related
- [[filesystem_as_database]] — the largest consumer of multi-object commit.
- [[configuration_and_policy]] — config changes as atomic, rollback-able transactions.
- [[package_system]] — atomic install/uninstall as a transaction.
- [[updates_and_hot_swap]] — upgrade as a transaction over migrated state.
- [[capability_model]] — atomic grant/revoke and graph-edge consistency.
