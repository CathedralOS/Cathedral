# Chapter 03: State Evolution And Live Migration

> Cathedral must carry state across changing formats and component artifacts;
> Omega supplies checked ordinary data and transformations, not a universal
> version container.

## The Legacy Model

Traditional systems scatter evolution across startup ladders, database scripts,
wire libraries, and service-manager hooks. None can prove that the old shape
was decoded under the intended format, that the transform establishes the new
invariants, or that a failed live cutover still has enough old state to resume.

## The Cathedral Model

Cathedral separates three histories:

- **durable/external formats** use explicitly named protocol shapes, stable
  field identities where a fluid codec needs them, and codec-specific unknown
  version policy;
- **runtime state transforms** are ordinary Omega machines from an explicit old
  type to an explicit new type; and
- **live replacement** is the operational phase protocol in
  [[updates_and_hot_swap]].

There is no `Versioned<T>`, `FileRecord.prev`, or compiler-owned history chain.
One runtime type can have independent disk, IPC, cache, and component-state
histories.

```omega
data FileRecordDiskV1 {
    1: bytes: InlineBytes;
    2: timestamp_millis: i64;
}

data FileRecordRuntime {
    body: ContentRef;
    created: DateTime;
}

machine migrate_file_v1(
    old: FileRecordDiskV1,
    out: &mut FileRecordRuntime
)
    ensures out in FileRecordRuntime::Valid
{
    out.body = ContentRef::from_inline(old.bytes);
    out.created = DateTime::from_millis(old.timestamp_millis);
}
```

Projects may organize such machines with an ordinary
`Upgradable<Old, New, Context>` trait. Cathedral can ship that trait in its
replacement framework; Omega does not privilege it.

## Durable Histories

Old durable formats remain ordinary, immutable schema declarations. A decoder
for several known formats returns an ordinary sum and an unknown-format result
according to the codec's policy. Stable field numbers permit compatible tagged
evolution within one fluid schema; a breaking format gets a new named shape.

The durable identity is a typed normalized schema/codec-plan identity plus
authored nominal metadata—not a moving runtime type name and not one global
version number. Publication checks predecessor compatibility and tombstones.

Construction and provenance stay separate. Tests may construct old shapes
directly; trusted install/load paths require the decoder-established provenance
domain before admitting them as captured state.

## Live-State Transformation

A live cutover normally transforms the currently installed artifact's owned
state into the candidate artifact's state. That is one explicit `(Old, New)`
edge for this operation, not a language claim that all histories are linear or
single-step.

When transformation needs clocks, device state, files, or other authority:

```text
effectful capture -> owned context -> replayable prepare -> atomic commit
```

Replayability requires more than an empty effects row: inputs are owned, output
is exclusive, shared/atomic observations are absent, and every callee promises
deterministic behavior.

The point-of-no-return law is strict:

- before commit, failure leaves the old component meaningfully resumable;
- preparation borrows old state or retains enough ownership to restore it;
- commit consumes the phase tokens only as installation becomes atomic; and
- destructive handoff is a separately declared non-rollbackable protocol with
  all fallible work complete first.

## Division Of Labor

Omega supplies ordinary types, sums, patterns, traits, domains, contracts,
linearity, effects, normalized identities, and compatibility/refinement
artifacts. Cathedral supplies format packages, migration topology, capture
policy, quiescence, installation, health policy, and rollback.

Cathedral is also the first consumer deciding where the reusable replacement
framework belongs: Cathedral-local code, a target-neutral Omega library, or an
irreducible runtime primitive. The implementation decides; verbosity alone
does not.

## Key Questions

- Should stale durable objects migrate lazily on load, eagerly during a store
  sweep, or under a per-schema policy?
- Which migrations promise losslessness or a separately checked inverse?
- How do migrations interact with content-addressed bodies shared by many
  records: migrate the reference, the content, or both?
- What route-plan law should a format package prove when every known old shape
  must reach the current runtime shape?

## Omega Leverage

- [Protocol Schemas And Serialization](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)
  supplies stable identity metadata and codec/layout contracts.
- [Evolution, Migration, And Replacement](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md)
  supplies the ordinary-mechanisms decomposition and point-of-no-return law.
- Omega domains name decoded provenance and new-state invariants.
- Ownership and linearity account for old state and phase tokens.

## Related

- [[filesystem_as_database]] — durable records and snapshots.
- [[updates_and_hot_swap]] — quiescence, installation, coexistence, rollback.
- [[memory_and_persistence]] — crash consistency during migration.
- [[ipc_and_service_invocation]] — protocol evolution across component edges.
