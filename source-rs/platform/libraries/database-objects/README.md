# Private object roots over immutable content

The [accepted copy contract](../../../../wiki/spec/storage/realms_and_objects.md)
requires independent edits on both sides of a copy. This implementation shares
content but duplicates hierarchy metadata. The contract now selects paths within
a supplied view for persisted references; this numbered-object implementation
does not yet provide named path lookup or stable persisted references. Names
and queryable media types belong to hierarchy metadata under that contract;
the records here do not yet carry those fields or provide type filtering.

[lib.rs](lib.rs) opens the experimental table and defines the bounds.
[records.rs](records.rs) owns key scoping and the fixed head, event, edge and
pin encodings. [objects.rs](objects.rs) numbers objects, keeps the tree and
implements the head-only operations: look up, list a parent's children, test
membership in a subtree, copy one object or a whole subtree, and delete.
[publication.rs](publication.rs) orders size/stale checks, preparation, writer
revalidation, immutable-record insertion and the head/event/edge/quota update
inside one immediate transaction. [reading.rs](reading.rs) pins a read
transaction and verifies the head's event, manifest and the requested chunks.
[collection.rs](collection.rs) pins revisions and runs bounded reachability
collection. No client IPC or driver behavior lives here.

A root is the unit the provider serves. Inside it, objects are numbered from one
and numbers are never reused, so a stale client holding a deleted number can never
reach a newer object. Every object has a parent, zero being the root itself, and
an edge record indexes each parent's children, so the root is a tree and any
object names the subtree beneath it. A parent exists before anything is placed
under it and an object never moves, so every parent's number is smaller than its
children's; that is what makes an ancestry walk terminate without a depth bound.
An object with children cannot be deleted. Each object has a head (current
revision, manifest digest, length, parent) and one event record per published
revision. Chunks and manifests are keyed by digest within the root and inserted
only when absent, so identical content is stored once however many objects or
revisions refer to it. Nothing is shared across roots: cross-root deduplication
and digest-based access are not introduced accidentally.

A copy is therefore cheap by construction: it allocates a number under the
chosen parent and writes a head, an event and an edge that reference the
source's manifest. The content is not read, hashed or rewritten, whatever its
size. A subtree copy does the same for the source and every descendant,
preserving the shape under new numbers, and is refused whole beyond
`COPY_LIMIT` objects. Editing either side afterwards leaves the other untouched
and keeps sharing every chunk the edit did not change. A realm made of many
objects is copied for the price of three small records per object.

The shared [content encoding](../content-layout/lib.rs) is extracted from the host
layout lab, preserving its exact BLAKE3 domains and inline/flat/tree manifest bytes.
This guest store uses only inline and flat forms. Tree construction remains in the
host lab until larger bounded guest workloads justify it.

The caller must supply a trusted root binding; this library is not an authority
validator. The [provider](../../services/storage/service.rs) obtains the
binding from its kernel-authenticated connection and uses `within` to confine a
request to the subtrees its scope stamps name. Publication callbacks observe
preparation and successful commit outside the writer; the Committing observation
runs inside the writer immediately before commit and must not reenter the store.

History is retained until collection, which the provider runs before every write
to a root and which clients may also request; the fixed logical quota can refuse
new publications first. Two persistent root-owned pin slots preserve selected
revisions of any object, and a pinned revision stays readable after its object is
deleted. Collection derives reachability from every current head and both pins
under one writer and removes at most eight records per immediate commit. A pass
reads only metadata: it marks chunks by the digests their live manifests name
and sweeps them through a key-only chunk index, never reading or re-hashing
chunk bytes, so a root holding programs costs a write no more than a root of
notes; reads authenticate chunks when they are used. Obsolete events are retired
before manifests and chunks so a pin between passes cannot resurrect a partially
collected revision. It frees engine pages for reuse; it does not shrink the file.

Accounting includes exact retained keys and values, including the ledger itself.
It excludes redb pages, allocator overhead and free-space reserves.

Inside `source-rs/`, `cargo test --locked -p cathedral-database-objects --features host`
covers refusals leaving no partial records, root privacy, a copy that shares every
chunk and costs only its head, event and edge, deletion releasing only unshared
content, number reuse never happening, listing by parent, subtree membership, a
subtree copy that preserves shape and shares content, refusal of a copy into
itself or beyond the object limit, non-empty deletion, inline/chunk boundaries,
corruption detection, writer revalidation after preparation, cold opening at every
collection boundary with live pins, and 600 replacements inside a fixed extent
beside unrelated data. See the [guest proof](../../../distribution/lab/objects/README.md)
and the [storage proposal](../../../../wiki/proposals/0002_dbfs_storage_foundation.md)
for evidence and unresolved production semantics.
