# Realms and objects

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-25 |

## Scope

This contract establishes independent-copy behavior, path-based persisted
references, queryable hierarchy metadata and the separation of hierarchy from
stored backing, following the owner's 2026-10-18 decisions. It does not select
internal object IDs, alias semantics or a production disk format.

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

## Names and media types are hierarchy metadata

Names belong to entries in the parent hierarchy. Declared media/content type
also belongs to the hierarchy metadata describing an entry, not to the shared
content backing as a global property. Independent entries may describe the
same bytes differently. Changing a name or declared type changes hierarchy
state without changing the referenced file content. Hierarchy records may
themselves be hashed and stored; their hashes can change on metadata edits.

Content type is an explicit structured field that the database can index and
filter. Finding entries by media type must not require parsing every filename
or opening every content body. This is the basis for efficient type-based
queries within the caller's authorized view, not a latency guarantee or a
choice of index layout or query language. Query results and counts must respect
metadata visibility; an index is not an alternate way to enumerate other realms.

A filename extension is a convention, not the source of truth for the declared
type. Importers, downloaders and explorers may infer a type from an extension or
content and write metadata when authorized. Synchronizing extensions and type
metadata is program policy, not an implicit storage rename operation. Unknown
or absent type information does not make an extension authoritative. The type
vocabulary and unknown-value encoding are not frozen by this contract.

Declared type describes intended interpretation; it neither proves that bytes
are valid nor grants execution or other authority. Consumers still validate
formats and operations. Entries may have both content and children: media type
does not impose a file-versus-folder split, and a matrix remains a program rather
than requiring its own structural storage kind.

## Paths and copied configuration

Ordinary persisted references, including matrix startup configuration, use
paths resolved within a supplied authorized storage view. The consuming
interface must define the path's base; resolving a path cannot escape the
granted view or fall back to an ancestor's or global namespace.

Copying preserves path strings unchanged. A copied environment resolves them
in its own supplied view. Copying an individual file does not bring along every
target its contents reference, rewrite its paths, or grant access to targets
outside the destination view. If the target is absent there, lookup fails.
The copy operation is not required to inspect arbitrary payloads or repair
their dependencies.

A path identifies a location, not an enduring object identity. Moving or
renaming its target may break a saved reference; the owner or application must
update the path if it wants to follow the move. A later lookup at the same path
resolves the entry currently there, subject to access and operation validation.
This does not require an already-open handle to retarget when a path changes.

A granted realm exposes no enclosing parent or universal root. The service may
keep private reverse hierarchy links without exposing them through that view.
[Change watches](change_watches.md) bind to the authorized object instance rather
than a path. Moving that watched root alone is invisible within its rooted view;
its old and new parents observe only relationships they are authorized to see.
Independent copies remain independent watch subjects despite shared backing.

Stable persisted IDs or GUIDs that follow moved objects are deferred. They are
not needed to implement this path model. Internal object numbers and transient
handles remain implementation mechanisms and do not grant ambient access.

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

Stable identity across views, aliases and cross-realm mutable sharing are
deferred. This contract does not mandate either new internal IDs for every
copied descendant or unchanged IDs resolved within each view. It does not
introduce a live inherited view with local overrides or an automatic
managed-update policy. Any future managed updates require separate authorized
enrollment and editing rules; ordinary clones do not enroll themselves.

Metadata encoding, type vocabulary, detailed path syntax and name comparison,
encryption domains, retention budgets and the complete snapshot and reclamation
protocols are outside this partial contract.

## Implementation evidence

The [Rust object store](../../../source-rs/platform/libraries/database-objects/README.md)
implements independent copies sharing immutable manifests and chunks. Its
subtree copy allocates new numbered objects and metadata per descendant; it
does not implement shared hierarchy roots. It now implements bounded exact
names and media types on edges, atomic index updates, scoped relative-path
lookup and indexed direct-child type filtering. The Rust wire protocol carries
these operations; extensions do not supply types or execution authority.
Named creation also publishes initial content, hierarchy metadata and indexes in
one transaction. A name collision refuses the whole operation. An ambiguous reply
is reconciled by path within the same authorized view before deciding on further
work; it never grants authority to replace whatever happens to occupy that name.
Host tests exercise independent edits, shared content, subtree copies, index
rollback and bounded reclamation with retained pins. Pins retain content
revisions, not complete historical hierarchy metadata. The desktop proof uses
names and a persisted launcher path; a renamed launcher works after the path is
updated, while a missing path returns to user selection.

The [storage protocol](../../../source-rs/platform/contracts/storage.rs)
exposes these bounded operations. The [storage proposal](../../proposals/0002_dbfs_storage_foundation.md)
and [filesystem design](../../design/part_4_storage/00_filesystem_as_database.md)
retain ownership of the broader candidate representation and design.
