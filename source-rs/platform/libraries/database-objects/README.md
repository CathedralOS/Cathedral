# Atomic private content records

[lib.rs](lib.rs) opens the experimental table and defines bounded admission.
[publication.rs](publication.rs) orders size/stale checks, preparation, writer
revalidation, immutable-record insertion, head/event/quota update and commit.
[reading.rs](reading.rs) pins a read transaction and verifies the referenced
manifest and requested payloads. [records.rs](records.rs) owns key scoping and
the fixed head/event representation. [retention.rs](retention.rs) pins/releases
revisions and orders bounded collection. No client IPC or driver behavior lives here.

The shared [content encoding](../content-layout/lib.rs) is extracted from the host
layout lab, preserving its exact BLAKE3 domains and inline/flat/tree manifest bytes.
This guest experiment uses only inline and flat forms. Tree construction remains
in the host lab until larger bounded guest workloads justify it. Each root's
records are independently prefixed, including chunks: cross-root deduplication
and digest-based access are not introduced accidentally.

The caller must supply a trusted root binding; this library is not an authority
validator. The [provider](../../services/storage/database/service.rs) obtains the
binding from its kernel-authenticated connection. Publication callbacks observe
preparation and successful commit for heap measurements; they run outside the
writer and do not define a public storage protocol.

History is retained until explicit collection; the fixed logical quota can refuse
new publications first. Two persistent root-owned pin slots preserve selected
revisions. A slot cannot replace an occupied pin, and release checks the revision.
These are lab retention semantics, not accepted backup/watcher guarantees.

Collection derives reachability from the current head and both pins under one
writer. It validates those three bounded closures, scans at most the 24 KiB root
record budget, and removes at most eight records per immediate commit. Obsolete
events are retired before manifests and chunks so a pin between passes cannot
resurrect a partially collected revision. Each pass adjusts the ledger atomically.
It frees engine pages for reuse; it does not shrink the file.

Accounting includes exact retained keys and values, including the ledger itself.
It excludes redb pages, allocator overhead and free-space reserves. Host checks
cold-open after every collection pass and perform 600 8 KiB replacements within
a fixed 1,056,768-byte extent containing 100 unrelated 4 KiB records. This is a
specific reuse trace, not a general full-volume liveness claim. See the
[guest proof](../../../distribution/lab/objects/README.md) and
[storage proposal](../../../../wiki/proposals/0002_dbfs_storage_foundation.md)
for evidence and unresolved production semantics.
