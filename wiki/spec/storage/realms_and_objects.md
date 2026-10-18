# Realms and objects

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-18 |

## Scope

This contract establishes independent-copy behavior and the separation of
hierarchy from stored backing, following the owner's 2026-10-18 decision.
It does not select logical object IDs, alias semantics or a production disk
format.

## Hierarchy and backing

The hierarchy describes the visible arrangement and references to content.
Backing stores the records and bytes that represent it. Hierarchy records may
themselves use the object store; that does not merge namespace relationships,
physical placement and access authority into one concept.

An ordinary copy is independent. Copy-on-write is the default sharing model
for persistent hierarchy and content: copies may share backing, but a write
through either view must preserve the other view's state. The original has no
special right to overwrite backing still used by its copy. A clone does not
follow later source edits or template updates merely because it was cloned.

An implementation may initially share immutable hierarchy records beneath
independent roots and replace records along changed paths. It need not eagerly
duplicate all backing. This is not a constant-time cloning guarantee: authority,
accounting, identity representation and durable publication have their own costs.
The physical sharing permitted here grants no new authority and does not
authorize deduplication across trust or encryption domains.

## Replacement, exclusive use and reclamation

The initial implementation direction is immutable backing with replacement
writes. Exclusive-use optimization is optional, not required copy behavior.
If backing is reused in place, the implementation must preserve all retained
views and its crash-consistency guarantees. One visible namespace reference
does not prove exclusive use: snapshots, retained history and active readers
may still depend on the old state. Concurrent acquisition must also be safe.
Changed content cannot remain associated with its previous content digest.

Backing is eligible for reclamation only when no retained view or other live
consumer requires it. This contract does not choose reference counting,
reachability tracing or a combination. An unreachable record awaiting cleanup
is not a promised recycle bin; recoverable trash must deliberately retain the
state it promises to recover.

## Exclusions

Logical identity across views, resolution of references inside copied program
state, aliases and cross-realm mutable sharing remain unspecified. This contract
does not mandate either new IDs for every copied descendant or unchanged IDs
resolved within each view. It does not introduce a live inherited view with
local overrides or an automatic managed-update policy.

Names, kinds, encryption domains, retention budgets and the complete snapshot
and reclamation protocols are outside this partial contract.

## Implementation evidence

The [Rust object store](../../../source-rs/platform/libraries/database-objects/README.md)
implements independent copies sharing immutable manifests and chunks. Its
subtree copy allocates new numbered objects and metadata per descendant; it
does not implement shared hierarchy roots or settle production identity.
Its existing host tests exercise independent edits, shared content, subtree
copies and bounded reclamation with retained pins.

The [storage protocol](../../../source-rs/platform/contracts/storage.rs)
exposes these bounded operations. The [storage proposal](../../proposals/0002_dbfs_storage_foundation.md)
and [filesystem design](../../design/part_4_storage/00_filesystem_as_database.md)
retain ownership of the broader candidate representation and design.
