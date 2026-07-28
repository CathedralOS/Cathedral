# Chapter 01: Transactions & Consistency

> Atomicity is an OS primitive, not a database feature: this chapter owns the one common mechanism by which a set of state transitions commits all-or-nothing.

## The Legacy Model

In a traditional OS, transactions live *inside* a database and nowhere else. Installing a package, applying an update, editing a config file, changing a permission, or moving files are all multi-step mutations with no atomicity — they half-complete, leaving a state no design intended. The canonical recovery is "delete the lockfile and run it again." Each subsystem reinvents a private, partial commit/rollback (dpkg's `--configure -a`, a `.bak` file, a journal replay), and none compose. There is no OS-level answer to "make these N changes atomically, or none of them."

## The Cathedral Model

A single transactional primitive that any subsystem can use to make a set of state transitions atomic, isolated, and rollback-able — exposed the same way for filesystem writes, package installs, upgrades, UI state, configuration changes, and capability grants/revocations. The trick is *not* to make the whole OS globally transactional (that way lies a planet-sized lock). It is to identify precisely **which** state transitions must be atomic, draw that boundary explicitly, and give it one shared commit/abort/compensation machinery. Outside those boundaries, the system stays loosely coupled and eventually consistent on purpose.

A transaction reads as a scope over participating state with a proven outcome:

```omega
transaction grant_and_record {
    participates cap_table, audit_log
    requires exclusive(cap_table)
    ensures  committed | rolled_back
}
```

### The decided mechanism

**Commit is an explicit runtime act, not magic.** Durable state becomes durable through an actual commit — a host/stdlib `store.commit(obj)` call (API spelling illustrative), or a declared transaction the compiler lowers to one. There is **no implicit RAII commit**, and reach to the `Storage` boundary service is only a static contract fact — it persists nothing by itself. ("No save" is a slogan; there is always a commit call.) The atomic unit is **one object's new version, written as one realm-log append** (CoW): mutate its fields in the RAM working copy, then commit. So **single-object atomicity is free** — co-locating the things that must commit together as fields of one aggregate (e.g. an outbox as a field of the account it guards) makes "both or neither" automatic, with no transaction primitive at all.

**A real transaction is needed only when state spans objects.** Within one realm: stage writes to N objects, path-copy to the realm root, commit the root once (one append) — same log, still cheap. **Across realms on one machine: the OS coordinates 2-phase commit** (reliable coordinator, no partition — genuinely easy; this is one of the few intrinsically-OS pieces). **Across machines: 2PC is wrong** (it blocks under partition) — use **sagas / compensation** ([[distributed_boundary]]).

**The API shape: no `begin`, one `commit` verb that dispatches.** Writes are plain mutations to a RAM copy-on-write working overlay — *not* syscalls. Because you already hold the participating realm capabilities and concurrency is optimistic (no upfront locks to acquire), nothing happens up front: you mutate, then `commit`, and the runtime dispatches on the write set — a single object (or one realm) is one log-append + one durability flush; cross-realm is 2-phase commit, whose two phases *are* the two flushes (prepare-durable, then commit-durable). Participants in a cross-realm commit are named by their specific realm-cap **slots** (the O(1) fd-like handle, [[capability_model]]), and those caps are **re-checked live at commit** — a revocation mid-transaction aborts it (optimistic on *authority*, not just data). A thin affine wrapper makes the uncommitted RAM overlay **abort through nonblocking edge cleanup**, so a trap or early return cannot publish a half-staged transaction; any rollback that must perform IO is an explicit operation. `commit` remains the only path to durability.

**Concurrency: snapshot isolation + optimistic.** CoW *is* MVCC — every reader gets a consistent point-in-time view of the object graph with no reader locks. Write-write conflicts use **optimistic first-committer-wins** (detect at commit, abort+retry or surface a merge), never lock-on-open ("file in use" is the wrong UX) and never silent last-write-wins (lost updates). Most data is uncontended, so it is invisible; genuine concurrent edits surface as a conflict to merge ([[distributed_boundary]]). This is the database playbook (Postgres/InnoDB MVCC; FoundationDB/Spanner optimistic), inherited because the store is a db-fs — conventional OSes punt this (Windows TxF was deprecated; file locks are the alternative).

**Irreversible effects are fenced OUT, not made to "participate."** Reach to world-touching services such as `Network` or `DeviceIo` cannot appear inside a commit, because commit code carries a `Storage`-only effect ceiling and Omega rejects any additional boundary-service reach — **the fence is a free consequence of the existing effect system, no new keyword or `[irreversible]` marker.** The irreversible act is handled *outside* the transaction by the **transactional-outbox pattern** (record a durable intent *inside* the commit; emit *after* it; carry an idempotency key for crash-replay) — a **userspace/stdlib pattern, not OS machinery.** The OS supplies only the atomic commit (which the filesystem needs anyway) and the compiler supplies only the fence; the resend/reconcile loop is irreducibly the app's. The db-fs's value here is just *no second WAL* (intent commits in the same realm-log append as the state) and *whole-system atomicity* (no private DB needed).

## Concerns & Design Space

- **Where the boundaries are.** The core design work: classifying state transitions into "must be atomic together" sets. Package install + capability grant is one transaction ([[package_system]], [[capability_model]]); a background telemetry write is not.
- **Multi-object commit.** The filesystem-as-database ([[filesystem_as_database]]) is the largest consumer: rename + content + metadata commit as one log entry.
- **Atomic grant/revoke.** A capability grant and its authority-graph edge, or a revoke and its sub-tree invalidation, must land together ([[capability_model]]).
- **Upgrade transactions.** An upgrade is a transaction whose participants include live state migration ([[updates_and_hot_swap]]); abort means roll back to the prior version cleanly.
- **Rollback & compensation.** Some effects reverse (state writes); some do not (a sent packet, a fired actuator) and need *compensation*, not naive undo.
- **Conflict detection & isolation.** Concurrent transactions on the same state: optimistic (detect-and-retry) vs. pessimistic (lock-and-block), and the isolation level each subsystem actually needs.
- **Distributed transactions / sagas.** Across the distributed boundary ([[filesystem_as_database]] sync, multiple devices), two-phase commit is often wrong; long-lived sagas with compensation usually fit better.
- **Zero value.** A zero transaction has no participants and commits as a no-op (valid-empty): it satisfies `committed` trivially and leaves state untouched, so an empty commit is coherent rather than an error ([[omega_substrate]]).

## Key Questions

- What is the smallest transactional API that serves filesystem, package, config, and capability subsystems without forcing them into one storage engine?
- Which transitions are *required* atomic vs. merely *convenient* atomic — and who is authorized to declare a new transactional boundary?
- **Irreversible effects — decided** (see mechanism): *forbidden inside* a transaction (the commit's `Storage`-only ceiling rejects `Network`/`DeviceIo` reach), and handled *after* commit by the userspace transactional-outbox pattern (durable intent in-commit, emit post-commit, idempotency key). Not OS machinery — a stdlib pattern + a free fence.
- **Isolation — decided:** snapshot isolation (CoW = MVCC, no reader locks) is the default; write-write conflict uses optimistic first-committer-wins (abort+retry / surface a merge), never lock-on-open, never silent last-write-wins. The database playbook, inherited because the store is a db-fs.

## Omega Leverage

- **Machines, states & transitions** model a transaction as an explicit state graph (`open -> staged -> committed | aborted`) the compiler can inspect — see Omega [States And Transitions](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md).
- **Effects** distinguish reversible from irreversible work: an effect ceiling tells the system which participants need compensation rather than rollback — see Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Domains** name the commit lifecycle (`Txn::Staged`, `Txn::Committed`) and let `requires`/`ensures` enforce that participants reach a terminal domain — see Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **Ownership / borrowing** supplies the isolation substrate: exclusive access is already a proven fact, so pessimistic isolation can lean on the borrow checker.
- Omega does not model distributed commit/saga coordination; that is OS runtime policy layered over the typed local primitive.

## Open Questions

- Can the borrow checker alone provide serializable isolation for in-memory participants, or is a runtime conflict detector always required?
- Is there one transaction manager, or per-domain managers that federate — and how is a cross-subsystem transaction coordinated without a global bottleneck?
- How long may a transaction hold participants before it blocks hot swap or starves other writers ([[updates_and_hot_swap]])?

## Related
- [[filesystem_as_database]] — the largest consumer of multi-object commit.
- [[configuration_and_policy]] — config changes as atomic, rollback-able transactions.
- [[package_system]] — atomic install/uninstall as a transaction.
- [[updates_and_hot_swap]] — upgrade as a transaction over migrated state.
- [[capability_model]] — atomic grant/revoke and graph-edge consistency.
