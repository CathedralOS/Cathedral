# Chapter 18: Filesystem as Database

> The filesystem is not a tree of byte-blobs; it is a durable, queryable object
> graph where every change is structured, observable, replayable, and permissioned.

## The Legacy Contract

Unix gives you `open`, `read`, `write`, `rename`, `stat`, `unlink`, and — if you
are lucky and on the right platform — `inotify`/`kqueue`/`FSEvents`. A file is an
opaque byte stream; a directory is a list of names; metadata is a fixed `stat`
struct plus best-effort xattrs. There are no transactions across files, no query
language, no history, no durable subscription, no schema, no provenance.
Atomicity is faked with write-to-temp-then-`rename`. Change notification is lossy
and drops events on overflow. "What changed, when, by whom, and can I replay it?"
has no answer. Applications rebuild the same machinery — SQLite, journals,
watchers, sync engines — atop an abstraction that actively fights them.

## What Cathedral Wants

A filesystem that *is* a database: a durable object graph with a transaction log
at its core. Files are records with typed metadata; directories are indexes and
views over those records. Every mutation is a structured, durable, ordered entry
in a log — so reads can be point-in-time snapshots, watchers become durable
subscriptions that never miss an event, and the past is replayable. Objects carry
content addresses (dedup, integrity), causal history (undo, audit), and
per-object capabilities (a directory listing is an *attenuated view*, not a raw
mode bit). The point is not a better ext4 — it is that **every change
is structured, durable, observable, replayable, permissioned, and queryable**,
which is exactly what robust file watching, app sync, backup, audit, undo, and
state migration all need underneath.

## Concerns & Design Space

- **Records and views.** Files as typed records; directories as indexes/views;
  a query layer ([[33_observability_and_introspection]]) over metadata rather
  than `find`/`stat` loops. Search/indexing is a first-class service.
- **The transaction log as the source of truth.** Journaling is exposed *as an
  API*: change data capture (CDC) is reading the log forward from an offset.
  Snapshots are log positions; rollback is replay to a position ([[19_transactions_and_consistency]]).
- **Durable subscriptions.** File watching becomes a resumable cursor over the
  log: a subscriber holds an offset, survives reboot, and is guaranteed every
  change in order with no overflow drops.
- **Atomic multi-object change.** A rename, a config write, and a content update
  commit as one transaction — the primitive lives in
  [[19_transactions_and_consistency]]; this chapter is its first heavy consumer.
- **Content addressing & dedup.** Object bodies keyed by content hash give cheap
  dedup, integrity, and snapshot sharing; names are mutable references into it.
- **Per-object capabilities.** A handle to an object is the authority over it; a
  directory view is an attenuated capability ([[03_capability_model]]), not a
  re-checked ACL.
- **Provenance.** Each record carries who/what/when wrote it ([[34_audit_compliance_provenance]]).
- **Schema evolution.** Records have typed, versioned shapes that migrate
  ([[21_versioned_state_and_migration]]).
- **Offline/online sync & conflict resolution.** Causal history makes merge and
  conflict detection a property of the log, not an afterthought.

## Key Questions

- What is the unit of a "record" — a whole file, a sub-object, or a typed
  fragment — and how does that interact with huge opaque blobs (video, disk
  images) that resist structure?
- Is the log global, per-volume, or per-subtree, and how is ordering defined
  across objects that commit together vs. independently?
- How does a familiar POSIX-shaped UX (paths, `ls`, editors) project onto an
  object graph without leaking the database underneath to users who don't want it?
- What is retained forever vs. compacted, and who pays for unbounded history?

## Omega Leverage

- **Versioned `data`** gives file records their typed historical shapes and a
  migration path for on-disk schema evolution — see Omega
  [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **`wire data`** gives the *at-rest and on-the-wire* record/log encoding stable
  field numbers and explicit compatibility, so old log entries stay decodable —
  see Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Domains** express object permission shades and validity classes
  (`Object::Readable`, `Object::Committed`, `Snapshot::Sealed`) on one handle
  type — see Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **Capabilities as values + authority flow** make a directory view a derived,
  attenuated handle and make provenance a recorded acquisition edge.
- Omega does not yet define a durable-log/subscription-cursor primitive; whether
  that is a runtime service or a language-level abstraction is open.

## Open Questions

- Can the log be the *only* source of truth (event-sourced) at OS scale, or is a
  materialized object store required for performance, with the log as a journal?
- How are durable subscriptions garbage-collected when a subscriber dies without
  unsubscribing — lease-based cursors ([[04_capability_lifecycle]])?
- What is the consistency boundary between content-addressed bodies and the
  mutable name graph during a partial-failure write?

## Related
- [[03_capability_model]] — per-object capabilities and attenuated directory views.
- [[19_transactions_and_consistency]] — the atomic-commit primitive this chapter consumes.
- [[21_versioned_state_and_migration]] — schema evolution for on-disk records.
- [[33_observability_and_introspection]] — metadata query and the log as a surface.
- [[34_audit_compliance_provenance]] — provenance recorded per change.
