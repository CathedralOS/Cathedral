# Private object roots over immutable content

The [accepted copy contract](../../../../wiki/spec/storage/realms_and_objects.md)
requires independent edits on both sides of a copy. This implementation shares
content but duplicates hierarchy metadata. Persisted references use relative
paths within a supplied view. Names and explicit media types live on hierarchy
edges, with indexes for exact name lookup and direct-child type queries.
Numbered IDs still address individual operations; stable cross-view IDs are deferred.

[lib.rs](lib.rs) opens the experimental table and defines the bounds.
[records.rs](records.rs) owns key scoping and the fixed head, event, edge and
pin encodings. [objects.rs](objects.rs) numbers objects, keeps the tree and
implements the head-only operations: look up, list a parent's children, test
membership in a subtree, copy one object or a whole subtree, and delete.
[hierarchy.rs](hierarchy.rs) owns metadata, name/type indexes, revision-checked
rename/retype, relative lookup and filtering. These changes commit atomically
with the head/event and exact retained-byte quota.
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
objects is copied for the price of small hierarchy records and their indexes.
Descendant names and types survive a subtree copy; the new root receives an
explicit new name or remains unnamed. No copied path inside content is rewritten.

Names and media types each occupy at most 32 UTF-8 bytes. Names compare exactly,
are unique among named siblings, and exclude slash, ASCII controls, `.` and `..`.
Empty fields preserve legacy unnamed/untyped entries. Paths are at most 256 bytes,
relative to an explicit base; `.` is allowed, parent traversal and absolute paths
are refused. Type filtering is exact and uses the scoped `(parent, type, id)`
index, not content sniffing or extensions. These are lab bounds, not a final ABI.
The existing `t` edge value carries the 64-byte metadata; `d` and `y` keys index
names and media types. Metadata changes increment the object's revision without
rewriting its content manifest. Pins retain content revisions, not historical
hierarchy metadata snapshots.

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

`create_named` uses the publication transaction for initial content, revision one,
hierarchy edge, exact name/type indexes and quota. Invalid metadata, collision or
capacity failure aborts the entire writer. `copy_named` similarly names a new
content-sharing object in the copy transaction. Neither overwrites a collision.
The runtime sends `CREATE_NAMED` with a 64-byte metadata prefix and initial content
in one sealed attachment (total at most 256 KiB). Ordinary PUBLISH retains its full
256 KiB content bound. An uncertain create reply requires name lookup and inspection,
not blind replacement; provider failure still retires the ambiguous stream.

Host checks interrupt staging and cold-open disk snapshots after every backend
write during named creation. Each recovered state contains either no new object
or the complete named content/index set, with an exact quota ledger. A retry after
an unobserved committed result collides without allocating another object.

History is retained until collection. Before mutations, the provider calls
`collect_under_pressure`: below a 1.5 MiB ledger it performs no writer/scan, and
otherwise runs one bounded pass. This leaves 512 KiB of the 2 MiB allowance as
publication headroom; exact quota admission still happens inside every writer.
Explicit `collect` is unconditional. Unpinned history has no retention guarantee;
the fixed logical quota can still refuse new publications before a backlog is
drained. Two persistent root-owned pin slots preserve selected
revisions of any object, and a pinned revision stays readable after its object is
deleted. Collection derives reachability from every current head and both pins
under one writer and removes at most eight records per immediate commit. A pass
reads only metadata: it marks chunks by the digests their live manifests name
and sweeps them through a key-only chunk index. It does not fetch or hash payload
entries; underlying engine pages can still contain neighboring payload data.
Work grows with live heads, manifests and distinct chunk references, so installed
programs do affect save cost. Memory does not: a pass marks one record kind at a
time inside one window of identities, and a window whose marks would exceed
`WORKSPACE_BYTES` (24 KiB, counted at 64 bytes per mark) is split in two and
marked again, down to 256 windows per kind. Events are windowed by the low byte
of their object number, manifests and chunks by the first digest byte. Heads are
read in batches of 32, never gathered whole. A window that still overflows at the
finest split returns `Error::Workspace` with nothing removed; it needs more than
384 live identities sharing one bucket byte, beyond what the root quota admits.
Chunk-index presence is validated window by window in ordered scans before any
deletions, avoiding repeated random lookups for shared chunks. Pinning uses the
same validation for its one revision before committing its new reference. Reads
authenticate chunk contents when they are used. Obsolete events are retired
before manifests and chunks so a pin between passes cannot resurrect a partially
collected revision; the event pass authenticates every head's event first. It
frees engine pages for reuse; it does not shrink the file. `Collection` reports
how many windows a pass computed and its largest marked set, so a provider can
observe when a root starts needing splits. `collect_records_within` takes an
explicit workspace for host tests and tools.
The exported `collect_records` applies that same bounded pass to a caller-owned
transaction. The offline stock-program updater uses it under quota pressure after
staging all stock and named replacement heads in host scratch space. The complete
result is quota-checked before publication; standalone migrations still enforce
their own final quota. Guest
`Store::collect` still runs one pass per transaction with unchanged semantics.

The lab allowance is 2 MiB per root, including installed code and nested snapshots.
Accounting includes exact retained keys and values, including the ledger itself.
It excludes redb pages, allocator overhead and free-space reserves.

Inside `source-rs/`, `cargo test --locked -p cathedral-database-objects --features host`
covers refusals leaving no partial records, root privacy, a copy that shares every
chunk and costs only its head, event and edge, deletion releasing only unshared
content, number reuse never happening, listing by parent, subtree membership, a
subtree copy that preserves shape and shares content, refusal of a copy into
itself or beyond the object limit, non-empty deletion, inline/chunk boundaries,
corruption detection, writer revalidation after preparation, cold opening at every
collection boundary with live pins, 600 replacements inside a fixed extent
beside unrelated data, a crowded root collected with a workspace too small for
one window per kind reaching exactly the unbounded result with every live read
and pin intact, and a workspace too small for one mark refusing whole. Hierarchy tests cover duplicate/stale rename rollback,
index updates, independent copy metadata, deletion, collection accounting and
reopening. See the [guest proof](../../../distribution/lab/objects/README.md)
and the [storage proposal](../../../../wiki/proposals/0002_dbfs_storage_foundation.md)
for evidence and unresolved production semantics.
