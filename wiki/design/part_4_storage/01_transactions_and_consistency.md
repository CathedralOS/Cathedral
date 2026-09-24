# Chapter 01: Transactions & Consistency

> Atomicity is an OS primitive, not a database feature. This chapter owns the one common mechanism by which a set of state transitions commits all-or-nothing.

## The Legacy Model

In a traditional OS, transactions live inside a database and nowhere else. Installing a package, applying an update, editing a config file, changing a permission, or moving files are all multi-step mutations with no atomicity. They half-complete and leave a state no design intended. The canonical recovery is "delete the lockfile and run it again." Each subsystem reinvents a private, partial commit/rollback (dpkg's `--configure -a`, a `.bak` file, a journal replay), and none of them compose. There is no OS-level answer to "make these N changes atomically, or none of them."

## The Cathedral Model

Cathedral has a single transactional primitive that any subsystem can use to make a set of state transitions atomic, isolated, and rollback-able. It is exposed the same way for filesystem writes, package installs, upgrades, UI state, configuration changes, and capability grants and revocations. The whole OS is not globally transactional; that way lies a planet-sized lock. The work is to identify which state transitions must be atomic, draw that boundary, and give it one shared commit/abort/compensation machinery. Outside those boundaries the system stays loosely coupled and eventually consistent on purpose.

A transaction reads as a scope over participating state with a proven outcome:

```omega
transaction grant_and_record {
    participates cap_table, audit_log
    requires exclusive(cap_table)
    ensures  committed | rolled_back
}
```

### The decided mechanism

Commit is a runtime act, not magic. Durable state becomes durable through an actual commit: a host/stdlib `store.commit(obj)` call (API spelling illustrative), or a declared transaction the compiler lowers to one. There is no implicit RAII commit. Reach to the `Storage` boundary service is only a static contract fact and persists nothing by itself. "No save" is a slogan; there is always a commit call.

The atomic unit is one object's new version, written as one realm-log append (copy-on-write). Mutate its fields in the RAM working copy, then commit. Single-object atomicity is therefore free. Co-locating the things that must commit together as fields of one aggregate, such as an outbox as a field of the account it guards, makes "both or neither" automatic with no transaction primitive at all.

A real transaction is needed only when state spans objects. Within one realm, stage writes to N objects, path-copy to the realm root, and commit the root once as one append. Same log, still cheap. Across realms on one machine, the OS coordinates two-phase commit. With a reliable coordinator and no partition this is easy, and it is one of the few intrinsically-OS pieces. Across machines, two-phase commit is wrong because it blocks under partition, so the answer there is sagas and compensation ([[distributed_boundary]]).

The API has no `begin` and one `commit` verb that dispatches. Writes are plain mutations to a RAM copy-on-write working overlay, not syscalls. You already hold the participating realm capabilities and concurrency is optimistic, so there are no upfront locks to acquire and nothing happens up front. You mutate, then `commit`, and the runtime dispatches on the write set. A single object, or one realm, is one log append plus one durability flush. A cross-realm write set is two-phase commit, whose two phases are the two flushes: prepare-durable, then commit-durable. Participants in a cross-realm commit are named by their specific realm-capability slots, the O(1) fd-like handle from [[capability_model]]. Those capabilities are re-checked live at commit, so a revocation mid-transaction aborts it. The transaction is optimistic on authority, not just on data. A thin affine wrapper makes the uncommitted RAM overlay abort through nonblocking edge cleanup, so a trap or early return cannot publish a half-staged transaction. Any rollback that must perform IO is a separate operation the code invokes itself. `commit` remains the only path to durability.

Concurrency is snapshot isolation plus optimistic conflict detection. Copy-on-write is MVCC: every reader gets a consistent point-in-time view of the object graph with no reader locks. Write-write conflicts use optimistic first-committer-wins. The conflict is detected at commit, and the loser aborts and retries or surfaces a merge. There is no lock-on-open, because "file in use" is the wrong UX, and no silent last-write-wins, because that loses updates. Most data is uncontended, so the mechanism is invisible; concurrent edits surface as a conflict to merge ([[distributed_boundary]]). This is the database playbook (Postgres and InnoDB MVCC; FoundationDB and Spanner optimistic), inherited because the store is a db-fs. Conventional OSes punt on it: Windows TxF was deprecated, and file locks are the alternative.

Irreversible effects are fenced out of a transaction rather than made to participate. Reach to world-touching services such as `Network` or `DeviceIo` cannot appear inside a commit, because commit code carries a `Storage`-only reach ceiling and Omega rejects any additional boundary-service reach. The fence is a consequence of the existing reach system; there is no new keyword and no `[irreversible]` marker. The irreversible act is handled outside the transaction by the **transactional-outbox pattern**: record a durable intent inside the commit, emit it after the commit, and carry an idempotency key for crash replay. This is a userspace/stdlib pattern, not OS machinery. The OS supplies only the atomic commit, which the filesystem needs anyway, and the compiler supplies only the fence. The resend/reconcile loop belongs to the app. The db-fs's value here is that there is no second WAL, since the intent commits in the same realm-log append as the state, and that atomicity is whole-system, so no private database is needed.

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
