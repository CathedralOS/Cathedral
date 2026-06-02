# Chapter 19: Transactions & Consistency

> Atomicity is an OS primitive, not a database feature: this chapter owns the one
> common mechanism by which a set of state transitions commits all-or-nothing.

## The Legacy Contract

In a traditional OS, transactions live *inside* a database and nowhere else.
Installing a package, applying an update, editing a config file, changing a
permission, or moving files are all multi-step mutations with no atomicity — they
half-complete, leaving a state no design intended. The canonical recovery is
"delete the lockfile, run it again, hope." Each subsystem reinvents a private,
leaky commit/rollback (dpkg's `--configure -a`, a `.bak` file, a journal replay),
and none compose. There is no OS-level answer to "make these N changes
atomically, or none of them."

## What Cathedral Wants

A single transactional primitive that any subsystem can use to make a set of
state transitions atomic, isolated, and rollback-able — exposed the same way for
filesystem writes, package installs, upgrades, UI state, configuration changes,
and capability grants/revocations. The trick is *not* to make the whole OS
globally transactional (that way lies a planet-sized lock). It is to identify
precisely **which** state transitions must be atomic, draw that boundary
explicitly, and give it one shared commit/abort/compensation machinery. Outside
those boundaries, the system stays loosely coupled and eventually consistent on
purpose.

A transaction reads as a scope over participating state with a proven outcome:

```omega
transaction grant_and_record {
    participates cap_table, audit_log
    requires exclusive(cap_table)
    ensures  committed | rolled_back
}
```

## Concerns & Design Space

- **Where the boundaries are.** The core design work: classifying state
  transitions into "must be atomic together" sets. Package install +
  capability grant is one transaction ([[22_package_system]], [[03_capability_model]]);
  a background telemetry write is not.
- **Multi-object commit.** The filesystem-as-database ([[18_filesystem_as_database]])
  is the largest consumer: rename + content + metadata commit as one log entry.
- **Atomic grant/revoke.** A capability grant and its authority-graph edge, or a
  revoke and its sub-tree invalidation, must land together ([[03_capability_model]]).
- **Upgrade transactions.** An upgrade is a transaction whose participants
  include live state migration ([[23_updates_and_hot_swap]]); abort means roll
  back to the prior version cleanly.
- **Rollback & compensation.** Some effects reverse (state writes); some do not
  (a sent packet, a fired actuator) and need *compensation*, not naive undo.
- **Conflict detection & isolation.** Concurrent transactions on the same state:
  optimistic (detect-and-retry) vs. pessimistic (lock-and-block), and the
  isolation level each subsystem actually needs.
- **Distributed transactions / sagas.** Across the distributed boundary
  ([[18_filesystem_as_database]] sync, multiple devices), two-phase commit is
  often wrong; long-lived sagas with compensation usually fit better.

## Key Questions

- What is the smallest transactional API that serves filesystem, package,
  config, and capability subsystems without forcing them into one storage engine?
- Which transitions are *required* atomic vs. merely *convenient* atomic — and
  who is authorized to declare a new transactional boundary?
- How do irreversible effects participate: are they forbidden inside a
  transaction, deferred until commit, or always paired with a compensator?
- What isolation guarantee is the OS-wide default, and where is it relaxed?

## Omega Leverage

- **Machines, states & transitions** model a transaction as an explicit state
  graph (`open -> staged -> committed | aborted`) the compiler can inspect — see
  Omega [States And Transitions](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md).
- **Effects** distinguish reversible from irreversible work: an effect ceiling
  tells the system which participants need compensation rather than rollback —
  see Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Domains** name the commit lifecycle (`Txn::Staged`, `Txn::Committed`) and
  let `requires`/`ensures` enforce that participants reach a terminal domain —
  see Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **Ownership / borrowing** supplies the isolation substrate: exclusive access is
  already a proven fact, so pessimistic isolation can lean on the borrow checker.
- Omega does not model distributed commit/saga coordination; that is OS runtime
  policy layered over the typed local primitive.

## Open Questions

- Can the borrow checker alone provide serializable isolation for in-memory
  participants, or is a runtime conflict detector always required?
- Is there one transaction manager, or per-domain managers that federate — and
  how is a cross-subsystem transaction coordinated without a global bottleneck?
- How long may a transaction hold participants before it blocks hot swap or
  starves other writers ([[23_updates_and_hot_swap]])?

## Related
- [[18_filesystem_as_database]] — the largest consumer of multi-object commit.
- [[20_configuration_and_policy]] — config changes as atomic, rollback-able transactions.
- [[22_package_system]] — atomic install/uninstall as a transaction.
- [[23_updates_and_hot_swap]] — upgrade as a transaction over migrated state.
- [[03_capability_model]] — atomic grant/revoke and graph-edge consistency.
