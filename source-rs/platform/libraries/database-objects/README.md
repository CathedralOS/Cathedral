# Private object roots over immutable content

[lib.rs](lib.rs) opens the experimental table and defines the bounds.
[records.rs](records.rs) owns key scoping and the fixed head, event and pin
encodings. [objects.rs](objects.rs) numbers objects and implements the head-only
operations: look up, list, copy and delete. [publication.rs](publication.rs)
orders size/stale checks, preparation, writer revalidation, immutable-record
insertion and the head/event/quota update inside one immediate transaction.
[reading.rs](reading.rs) pins a read transaction and verifies the head's event,
manifest and the requested chunks. [collection.rs](collection.rs) pins revisions
and runs bounded reachability collection. No client IPC or driver behavior lives here.

A root is the unit the matrix delegates. Inside it, objects are numbered from one
and numbers are never reused, so a stale client holding a deleted number can never
reach a newer object. Each object has a head (current revision, manifest digest,
length) and one event record per published revision. Chunks and manifests are
keyed by digest within the root and inserted only when absent, so identical
content is stored once however many objects or revisions refer to it. Nothing is
shared across roots: cross-root deduplication and digest-based access are not
introduced accidentally.

A copy is therefore cheap by construction: it allocates a number and writes a
head and an event that reference the source's manifest. The content is not read,
hashed or rewritten, whatever its size. Editing either object afterwards leaves
the other untouched and keeps sharing every chunk the edit did not change.

The shared [content encoding](../content-layout/lib.rs) is extracted from the host
layout lab, preserving its exact BLAKE3 domains and inline/flat/tree manifest bytes.
This guest store uses only inline and flat forms. Tree construction remains in the
host lab until larger bounded guest workloads justify it.

The caller must supply a trusted root binding; this library is not an authority
validator. The [provider](../../services/storage/service.rs) obtains the
binding from its kernel-authenticated connection. Publication callbacks observe
preparation and successful commit outside the writer; the Committing observation
runs inside the writer immediately before commit and must not reenter the store.

History is retained until collection, which the provider runs before every write
to a root and which clients may also request; the fixed logical quota can refuse
new publications first. Two persistent root-owned pin slots preserve selected
revisions of any object, and a pinned revision stays readable after its object is
deleted. Collection derives reachability from every current head and both pins
under one writer, scans at most the 24 KiB root record budget, and removes at most
eight records per immediate commit. Obsolete events are retired before manifests
and chunks so a pin between passes cannot resurrect a partially collected revision.
It frees engine pages for reuse; it does not shrink the file.

Accounting includes exact retained keys and values, including the ledger itself.
It excludes redb pages, allocator overhead and free-space reserves.

Inside `source-rs/`, `cargo test --locked -p cathedral-database-objects --features host`
covers refusals leaving no partial records, root privacy, a copy that shares every
chunk and costs only its head and event, deletion releasing only unshared content,
number reuse never happening, inline/chunk boundaries, corruption detection,
writer revalidation after preparation, cold opening at every collection boundary
with live pins, and 600 replacements inside a fixed extent beside unrelated data.
See the [guest proof](../../../distribution/lab/objects/README.md) and the
[storage proposal](../../../../wiki/proposals/0002_dbfs_storage_foundation.md)
for evidence and unresolved production semantics.
