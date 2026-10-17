# Atomic private content records

[lib.rs](lib.rs) opens the experimental table and defines bounded admission.
[publication.rs](publication.rs) orders size/stale checks, preparation, writer
revalidation, immutable-record insertion, head/event/quota update and commit.
[reading.rs](reading.rs) pins a read transaction and verifies the referenced
manifest and requested payloads. [records.rs](records.rs) owns key scoping and
the fixed head/event representation. No client IPC or driver behavior lives here.

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

History is retained until the fixed logical quota refuses new publications.
Accounting includes exact retained keys and values, including the ledger itself.
It excludes redb pages, allocator overhead and free-space reserves. There is no
collection or full-volume liveness claim. See the
[guest proof](../../../distribution/lab/objects/README.md) and
[storage proposal](../../../../wiki/proposals/0002_dbfs_storage_foundation.md)
for evidence and unresolved production semantics.
