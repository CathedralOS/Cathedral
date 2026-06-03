# Chapter 00: Filesystem as Database

> The filesystem is one content-addressed object graph, partitioned into capability-rooted realms with no global root, where every change is structured, durable, observable, replayable, and permissioned.

## The Legacy Model

Two separate failures, both structural.

First, there is one global tree. Unix fuses system and user data into a single hierarchy under `/`, so `/home`, `/etc`, `/usr`, and `/bin` are siblings. The user has to learn which directories are theirs, a stray delete can reach system files, and the OS cannot relocate or restructure its own files without breaking paths that apps hardcoded. System data should not be in the user's hierarchy at all.

Second, the API is byte-level. Unix gives you `open`, `read`, `write`, `rename`, `stat`, `unlink`, and, depending on the platform, `inotify`/`kqueue`/`FSEvents`. A file is an opaque byte stream; a directory is a list of names; metadata is a fixed `stat` struct plus best-effort xattrs. There are no transactions across files, no query language, no history, no durable subscription, no schema, no provenance. Atomicity is approximated with write-to-temp-then-`rename`, change notification is lossy and drops events on overflow, and "what changed, when, by whom, and can I replay it?" has no answer. Applications rebuild the same machinery (SQLite, journals, watchers, sync engines) atop an abstraction that does not provide it.

## The Cathedral Model

A filesystem that *is* a database: a durable object graph with a transaction log at its core. Files are records with typed metadata; directories are indexes and views over those records. Every mutation is a structured, durable, ordered entry in a log, so reads can be point-in-time snapshots, watchers become durable subscriptions that never miss an event, and the past is replayable. Objects carry content addresses (dedup, integrity), causal history (undo, audit), and per-object capabilities (a directory listing is an *attenuated view*, not a raw mode bit). The point is not a better ext4; it is that **every change is structured, durable, observable, replayable, permissioned, and queryable**, which is exactly what robust file watching, app sync, backup, audit, undo, and state migration all need underneath.

The other half of the redesign is that there is no global root.

### Realms: there is no global root

The store is partitioned into **realms**: independent rooted namespaces, each reachable only by holding a capability to its root.

- The **user realm** is the user's entire visible world. It is "the computer" from their seat.
- The **system realm** is a separate tree the user holds no capability to, so system files are not protected-but-visible; they are absent from the user's namespace entirely. That is strictly stronger than `chmod 700 /etc`, which still lets you see and name the thing.
- **Per-app realms** give each app its own private root; an app sees only its realm by default.
- **Shared and tenant realms** are explicitly granted spaces ([[multi_user_and_org_control]]).

Resolution is relative to a root you hold, not to a global `/`. A bare path names a node in your default realm, so the same path string can name different objects in different realms. That is not a collision; it is the consequence of decoupled roots. You only cross into another realm by naming it explicitly (a qualified name such as `system:/...`, which is a resolver directive selecting which root to start from) and holding a capability to it. This is naming-as-authority ([[naming_and_discovery]]) and the capability model ([[capability_model]]) applied to storage.

This is **not** "virtual drives." Drive letters got the named-root *syntax* right and the semantics wrong: they are ambient (every process sees `C:`), they imply a physical volume, and the user juggles them. A realm is capability-gated (you only have the roots you hold), decoupled from physical storage (realms are logical partitions over one shared store), and invisible to the user (they see only their own). A realm is a named root you hold a capability to, nothing more.

### One object graph underneath

Realms are not separate disks. Underneath every realm is one global, content-addressed object store: bodies keyed by hash, nodes with parent and child edges, deduped across realms. A realm is just a distinguished **root node** in that graph. A realm's top-level objects have the realm node as their parent, so an object's realm membership is *structural*: it is which root you reach by walking parent edges. There is no per-file realm tag; membership is ancestry. An optional cached `realm_root_id` per object turns a realm check into a field read instead of a walk.

The parent chain terminates at the realm root: an ordinary capability can walk *down* a realm but not *up* past its root. Above all realm roots sits exactly one privileged node, the **realm registry**, whose children are the realms. Traversing into it needs the realm-authority capability, held by the installer, backup tooling, and factory reset, not by ordinary principals. So there is still one global thing, but it is an *authority* almost nobody holds, not a *namespace* everyone shares.

### Cross-realm access and the honest cost

Cross-realm sharing is a capability-gated edge, not a global path. When an app needs a user file, something mints a capability into the user realm for that one object: that is the file picker as an authority mint ([[human_permission_ux]]). The distinction to keep explicit, which iOS and Android both blur, is app-private storage (the app realm) versus user-facing documents (the user realm): a file an app saves "for the user" lands in the user realm via a granted capability, not buried in the app's container.

The tradeoff, stated plainly: a global root is a composability win, realms are an isolation win, and you cannot fully have both. A bare path is no longer a universal address, so sharing across realms costs a capability plus a qualified name instead of passing a string, and a persisted reference must record realm-plus-path, not a bare path. A global object store still exists underneath, so realms partition *naming and authority*, not storage. And "see everything" for admin, backup, or search needs a meta-capability instead of being free.

## Concerns & Design Space

- **Records and views.** Files as typed records; directories as indexes/views; a query layer ([[observability_and_introspection]]) over metadata rather than `find`/`stat` loops. Search/indexing is a first-class service, scoped to the realms you hold.
- **The transaction log as the source of truth.** Journaling is exposed *as an API*: change data capture is reading the log forward from an offset. Snapshots are log positions; rollback is replay to a position ([[transactions_and_consistency]]). The log is likely per-realm.
- **Durable subscriptions.** File watching becomes a resumable cursor over the log: a subscriber holds an offset, survives reboot, and is guaranteed every change in order with no overflow drops.
- **Atomic multi-object change.** A rename, a config write, and a content update commit as one transaction; the primitive lives in [[transactions_and_consistency]] and this chapter is its first heavy consumer.
- **Content addressing & dedup.** Object bodies keyed by content hash give cheap dedup, integrity, and snapshot sharing across realms; names are mutable references into it.
- **Realm membership by ancestry.** Membership is the root the parent chain ends at, not a per-file tag; the optional cached `realm_root_id` is a denormalization, and a cross-realm move invalidates that cache over the moved subtree.
- **The system realm is swappable.** It wants to be immutable and content-addressed so the OS can version and roll it back independently of the user realm ([[versioned_state_and_migration]], [[updates_and_hot_swap]]); reinstall swaps the system realm and leaves the user realm untouched.
- **Per-object capabilities.** A handle to an object is the authority over it; a directory view is an attenuated capability ([[capability_model]]), and a realm root is just the top-most such capability.
- **Provenance.** Each record carries who/what/when wrote it ([[audit_compliance_provenance]]).
- **Schema evolution.** Records have typed, versioned shapes that migrate ([[versioned_state_and_migration]]).
- **Offline/online sync & conflict resolution.** Causal history makes merge and conflict detection a property of the log, not an afterthought.

## Key Questions

- What is the unit of a "record" (a whole file, a sub-object, or a typed fragment), and how does that interact with huge opaque blobs (video, disk images) that resist structure?
- What is the right realm granularity: one realm per app, or coarser, and who is allowed to mint a realm before it becomes fragmentation ([[governance_and_extension_boundaries]])?
- Is the log global, per-realm, or per-subtree, and how is ordering defined across objects that commit together vs. independently?
- A shared object reachable from two realms has no single home; is its "home realm" its creator, with others reaching it by a cross-realm edge, or is realm membership genuinely multi-valued?
- What is retained forever vs. compacted, and who pays for unbounded history?

## Omega Leverage

- **Capabilities as values + authority flow** make a realm root a held capability, a directory view a derived attenuation of it, and provenance a recorded acquisition edge. An object's realm is the root its parent chain ends at.
- **Versioned `data`** gives file records typed historical shapes and a migration path for on-disk schema evolution, and is what makes the system realm rollback-able. See Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **`wire data`** gives the at-rest and on-the-wire record/log encoding stable field numbers and explicit compatibility, so old log entries stay decodable. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Domains** express object permission shades and validity classes (`Object::Readable`, `Object::Committed`, `Snapshot::Sealed`) on one handle type. See Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- Omega does not yet define a durable-log/subscription-cursor primitive; whether that is a runtime service or a language-level abstraction is open.

## Open Questions

- Can the log be the *only* source of truth (event-sourced) at OS scale, or is a materialized object store required for performance, with the log as a journal?
- How are durable subscriptions garbage-collected when a subscriber dies without unsubscribing: lease-based cursors ([[capability_lifecycle]])?
- What is the consistency boundary between content-addressed bodies and the mutable name graph during a partial-failure write?
- How does a persisted cross-realm reference stay valid across a system-realm upgrade that relocates the target?

## Related
- [[capability_model]] — per-object capabilities, realm roots, and attenuated directory views.
- [[naming_and_discovery]] — realm-qualified names and resolution as a security primitive.
- [[human_permission_ux]] — the file picker as the cross-realm capability mint.
- [[multi_user_and_org_control]] — tenants as realms.
- [[transactions_and_consistency]] — the atomic-commit primitive this chapter consumes.
- [[versioned_state_and_migration]] — schema evolution and the swappable system realm.
- [[audit_compliance_provenance]] — provenance recorded per change.
