# Chapter 03: Versioned State & Live Migration

> This chapter owns the data/state-shape continuity primitive: the typed mechanism by which live state survives a change to its shape.

## The Legacy Model

On a traditional OS, a software update is "stop the thing, replace its files, start it again," with no check that the state on disk is still compatible. State continuity is informal: a hand-written upgrade script, an `if version < N` ladder in startup code, or a database migration tool the OS itself knows nothing about. When old and new shapes disagree, the failure modes are silent corruption, crash-on-load, or a one-way migration that cannot be rolled back. There is no typed notion of "old shape," no checked transform from old to new, and no way to prove a migration preserves the invariants the new code depends on.

## The Cathedral Model

Historical shapes coexist as **named versions** of a typed `data`, and a migration is **typed code** — a migration machine — that transforms an old shape into the current one under explicit effect, ownership, and invariant obligations. Migrations compose along a chain (`v1 -> v2 -> current`), so state from any known era can be carried forward by stitching known steps. Crucially, this chapter owns *shape continuity only*. The **operational** act of updating a running system — reaching quiescence, draining work, rolling upgrades, dependency planning, and operational rollback — belongs to [[updates_and_hot_swap]]. We define the typed transform; that chapter decides when it runs and on whose schedule.

This is a near-direct application of Omega's versioned-data model:

```omega
machine FileRecord::from_v1(old: FileRecord::v1, out: &mut FileRecord)
satisfies RuntimeMigratable<FileRecord::v1, FileRecord>
effects alloc
requires exclusive(old)
ensures FileRecord::invariants(out)
{
    out.body = ContentRef::from_inline(old.bytes);
    out.created = old.timestamp;
}
```

## Concerns & Design Space

- **Versioned data layouts.** Named historical shapes the compiler still type-checks, so old in-memory or on-disk state stays visible to the language rather than being reinterpreted as the current shape.
- **Schema evolution vs. wire evolution.** Runtime shape migration and external protocol/format evolution are *different obligations*. Wire compatibility (stable field numbers, reserved retirements, unknown-field policy) belongs to Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md); keep the two from being conflated.
- **Migration fallibility & rollback.** Whether a migration is infallible or may fail — and if it can fail, what the abort/rollback story is — must be explicit, feeding the transactional model ([[transactions_and_consistency]]).
- **Version matching at boundaries.** Places that can legitimately hold mixed versions: hot-swap state stores ([[updates_and_hot_swap]]), saved snapshots ([[filesystem_as_database]]), replicated state, and persisted memory ([[memory_and_persistence]]). Normal current-version code should not pay a version tag everywhere.
- **Chain composition & gaps.** A complete chain upgrades any old era automatically; a missing step makes the upgrade unavailable, not silently lossy.
- **Where migrated state lives.** Filesystem records, config, capabilities, and IPC protocol versions ([[ipc_and_service_invocation]]) are all migration participants sharing this one primitive.

## Key Questions

- How are version identifiers spelled and ordered (numeric, semantic, arbitrary), and how does the runtime select a chain when several paths exist?
- How does a migration declare fallibility and rollback so a failed migration is a clean abort, not a half-migrated object?
- What access guarantee does a migration require — exclusive, frozen, or quiescent — and who establishes it before the transform runs?
- At a mixed-version boundary, how is the version tag carried so that current-era code stays untagged and fast?

## Omega Leverage

- **Versioned `data` + migration machines** are the spine of this chapter — lean on them directly; migrations are ordinary typed machines with effect/ownership/ invariant contracts and compose along a chain. See Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **`wire data`** carries the *external* compatibility axis distinct from runtime migration — see Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Domains** name a value's version-validity and post-migration invariants (`FileRecord::Migrated`, `State::CurrentEra`) so `ensures` can demand them — see Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **Ownership / borrowing** supplies the `exclusive(old)` / frozen-access facts a migration needs; the swap-time obligations themselves are owned by [[updates_and_hot_swap]].

## Open Questions

- How are migrations themselves versioned and stored so an old snapshot found years later can still locate the chain that brings it current?
- Can a migration be proven *lossless* or *reversible* in the type system, or is reversibility always an explicit, separately-written inverse migration?
- How do migrations interact with content-addressed bodies that many records share ([[filesystem_as_database]]) — migrate the reference or the content?

## Related
- [[filesystem_as_database]] — snapshots and records as versioned, migratable state.
- [[updates_and_hot_swap]] — the operational act of running these migrations live.
- [[memory_and_persistence]] — persisted state that must match the running shape.
- [[ipc_and_service_invocation]] — protocol versions migrated at service boundaries.
