# Chapter 03: Versioned State & Live Migration

> This chapter owns the data/state-shape continuity primitive: the typed mechanism by which live state survives a change to its shape.

## The Legacy Model

On a traditional OS, a software update is "stop the thing, replace its files, start it again," with no check that the state on disk is still compatible. State continuity is informal: a hand-written upgrade script, an `if version < N` ladder in startup code, or a database migration tool the OS itself knows nothing about. When old and new shapes disagree, the failure modes are silent corruption, crash-on-load, or a one-way migration that cannot be rolled back. There is no typed notion of "old shape," no checked transform from old to new, and no way to prove a migration preserves the invariants the new code depends on.

## The Cathedral Model

A live component is always at its **last-installed version**, so a shape change is a **single step** — `prev -> current` — expressed as one typed migration carrying explicit effect, ownership, and invariant obligations. There is no multi-version chain at runtime: a component that skipped releases replays the single steps in sequence or refuses, never dispatching over many live eras at once. Persisted state that may be found *many* versions stale — a snapshot on disk, a replicated record — is deliberately **not** this chapter's problem: that is external, self-versioning data, and it belongs to `wire data` ([Omega ch21](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)). This chapter owns the *in-memory shape transform*; the **operational** act — reaching quiescence, draining, rolling upgrades, rollback — belongs to [[updates_and_hot_swap]]. We define the typed transform; that chapter decides when it runs and on whose schedule.

The transform is one Omega trait — `Upgradable<Old, New, Context = Nothing>` ([Omega ch22](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md)). Its context-free form is `migrate`:

```omega
machine FileRecord::migrate(old: FileRecord.prev, out: &mut FileRecord)
    satisfies Upgradable<FileRecord.prev, FileRecord>
    requires exclusive(old)
    effects  alloc
    ensures  out in FileRecord::Valid
{
    out.body    = ContentRef::from_inline(old.bytes);
    out.created = old.timestamp;
}
```

Because the trait carries `ensures out in New::Valid`, the migration is *forced* to re-establish the new shape's invariant domain — there is no way to install a half-built successor. When old state alone cannot build the new shape, the missing facts are captured first, by an effectful machine, into a private capture-only value passed as the `Context`: IO becomes data before the pure transform ([[updates_and_hot_swap]], Omega ch22). The identity that says *which* `prev` this is comes from a content hash of the old layout recorded in the build lockfile, not a hand-written `v1` — editing a shipped shape drifts its hash and is a compile error.

## Concerns & Design Space

- **Versioned data layouts.** The previous shape stays a named, type-checked `data` the compiler still understands, so old in-memory state is migrated by a typed transform rather than reinterpreted in place as the current shape.
- **Schema evolution vs. wire evolution.** This is the load-bearing split. Runtime *in-memory* shape migration (single-step, this chapter) and external *at-rest / over-the-wire* evolution (multi-version, stable field numbers, reserved retirements, unknown-field policy) are different obligations — the latter is Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md). A record that outlives many versions on disk is wire data; the live upgrade of the component that loads it is this chapter.
- **Migration fallibility & rollback.** The pure transform is the infallible part; any fallible work (IO, allocation that can starve) is pushed into the capture step that runs *before* `old` is mutated, so a failed upgrade aborts as "did nothing" ([[transactions_and_consistency]]). A *committed* but lossy migration is reversible only by a separately-written inverse, not for free.
- **Single-step, not version-matching.** Because live state is always current, normal code never pays a version tag: there is no mixed-era dispatch in memory. Mixed versions exist only where data is persisted or replicated ([[filesystem_as_database]], [[memory_and_persistence]]) — i.e. as wire data — and the tag lives there, not on the hot path.
- **Where migrated state lives.** In-memory component state is the single-step case here; filesystem records, config, and IPC protocol surfaces that persist or cross a boundary ([[ipc_and_service_invocation]]) are the wire-data case. The two share the *goal* (continuity) but not the mechanism.
- **Zero value.** ZII sets the floor: the zero state is a valid inhabitant of the current shape (valid-empty), so "no state yet" needs no special case, and a shape change whose new fields are all zero-valued is a `migrate` with an empty body rather than hand-written code ([[omega_substrate]]).

## Key Questions

- How is a shape's identity pinned (content hash of its layout), and how is the single `prev` it upgrades from selected, given there is no chain to walk?
- How does the capture/transform split declare fallibility so a failed upgrade is a clean abort before `old` is touched, not a half-migrated object?
- What access guarantee does the transform require — exclusive, frozen, or quiescent — and who establishes it before it runs ([[updates_and_hot_swap]])?
- At a persistence/replication boundary, how does wire data carry the version so in-memory current-era code stays untagged and fast?

## Omega Leverage

- **`Upgradable<Old, New, Context>` + the single-step migration** are the spine of this chapter — one typed machine with effect/ownership/invariant contracts, resolved by the `(Old, New)` type, not a chain. See Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md).
- **`wire data`** carries the *external, multi-version* compatibility axis this chapter delegates to — see Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md).
- **Domains** name the new shape's post-migration invariant (`FileRecord::Valid`, `State::CurrentEra`) so the trait's `ensures out in New::Valid` has something concrete to demand — see Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md). (Provenance — "this was produced by capture" — is *not* a domain; it is a private, capture-only type, since a value-predicate cannot prove who built a value.)
- **Ownership / borrowing** supplies the `exclusive(old)` fact the transform needs; the swap-time obligations themselves are owned by [[updates_and_hot_swap]].

## Open Questions

- A snapshot found years later is wire data, read by wire-compatibility rules — but at what point does the OS *upgrade* its in-memory form, lazily on load or eagerly on mount ([[filesystem_as_database]])?
- Can a migration be proven *lossless* or *reversible* in the type system, or is reversibility always an explicit, separately-written inverse ([[updates_and_hot_swap]] lossy-rollback residue)?
- How do migrations interact with content-addressed bodies that many records share ([[filesystem_as_database]]) — migrate the reference or the content?

## Related
- [[filesystem_as_database]] — snapshots and records as versioned, migratable state.
- [[updates_and_hot_swap]] — the operational act of running these migrations live.
- [[memory_and_persistence]] — persisted state that must match the running shape.
- [[ipc_and_service_invocation]] — protocol versions migrated at service boundaries.
