# Chapter 00: Filesystem as Database

> The filesystem is one content-addressed object graph, partitioned into capability-rooted realms with no global root, where every change is structured, durable, observable, replayable, and permissioned.

## The Legacy Model

Two separate failures, both structural.

First, there is one global tree. Unix fuses system and user data into a single hierarchy under `/`, so `/home`, `/etc`, `/usr`, and `/bin` are siblings. The user has to learn which directories are theirs, a stray delete can reach system files, and the OS cannot relocate or restructure its own files without breaking paths that apps hardcoded. System data should not be in the user's hierarchy at all.

Second, the API is byte-level. Unix gives you `open`, `read`, `write`, `rename`, `stat`, `unlink`, and, depending on the platform, `inotify`/`kqueue`/`FSEvents`. A file is an opaque byte stream; a directory is a list of names; metadata is a fixed `stat` struct plus best-effort xattrs. There are no transactions across files, no query language, no history, no durable subscription, no schema, no provenance. Atomicity is approximated with write-to-temp-then-`rename`, change notification is lossy and drops events on overflow, and "what changed, when, by whom, and can I replay it?" has no answer. Applications rebuild the same machinery (SQLite, journals, watchers, sync engines) atop an abstraction that does not provide it.

## The Cathedral Model

A filesystem that *is* a database: a durable object graph with a transaction log at its core. Files are records with typed metadata; directories are indexes and views over those records. Every mutation is a structured, durable, ordered entry in a log, so reads can be point-in-time snapshots, watchers become durable subscriptions that never miss an event, and the past is replayable. Objects carry content addresses (dedup, integrity), causal history (undo, audit), and per-object capabilities (a directory listing is an *attenuated view*, not a raw mode bit). The point is that **every change is structured, durable, observable, replayable, permissioned, and queryable**, which is exactly what robust file watching, app sync, backup, audit, undo, and state migration all need underneath.

The other half of the redesign is that there is no global root.

### Realms: there is no global root

The store is partitioned into **realms**: independent rooted namespaces, each reachable only by holding a capability to its root.

- The **user realm** is the user's entire visible world. It is "the computer" from their seat.
- The **system realm** is a separate tree the user holds no capability to, so system files are not protected-but-visible; they are absent from the user's namespace entirely. That is strictly stronger than `chmod 700 /etc`, which still lets you see and name the thing.
- **Per-app realms** give each app its own private root; an app sees only its realm by default.
- **Shared and tenant realms** are explicitly granted spaces ([[multi_user_and_org_control]]).

Resolution is relative to a root you hold, not to a global `/`. A bare path names a node in your default realm, so the same path string can name different objects in different realms. That is not a collision; it is the consequence of decoupled roots. You only cross into another realm by naming it explicitly (a qualified name such as `system:/...`, which is a resolver directive selecting which root to start from) and holding a capability to it. This is naming-as-authority ([[naming_and_discovery]]) and the capability model ([[capability_model]]) applied to storage.

Concretely, each principal carries a small **resolution environment**: a private table binding realm names (`system:`, `user:`, and so on) to actual root nodes. The name is shared vocabulary; the binding is private. That table is part of an instance's initial capability set at spawn ([[component_model]]), and the host chooses what each name points at. The host user's `system:` is bound to the real system root, while a sandboxed app's `system:` can be bound to an overlay, a view, or a synthetic root. So the same `system://...` string resolves to different objects for different principals, and an unmodified app that hardcodes `system://...` is sandboxed simply by binding its `system:` to a different root, the way a container rebinds `/`. The host owns every binding because it owns the tree, so it can resolve the real root, the app's root, and exactly where they diverge; the app holds only its own binding and cannot reach past it. This is Plan 9 per-process namespaces made capability-native; what can sit behind a bound root is the subject of the virtual-realms section below.

This is **not** "virtual drives." Drive letters got the named-root *syntax* right and the semantics wrong: they are ambient (every process sees `C:`), they imply a physical volume, and the user juggles them. A realm is capability-gated (you only have the roots you hold), decoupled from physical storage (realms are logical partitions over one shared store), and invisible to the user (they see only their own). A realm is a named root you hold a capability to, nothing more.

### One object graph underneath

Realms are not separate disks. Underneath every realm is one global, content-addressed object store: bodies keyed by hash, nodes with parent and child edges, deduped across realms. A realm is just a distinguished **root node** in that graph. A realm's top-level objects have the realm node as their parent, so an object's realm membership is *structural*: it is which root you reach by walking parent edges. There is no per-file realm tag; membership is ancestry. An optional cached `realm_root_id` per object turns a realm check into a field read instead of a walk.

The parent chain terminates at the realm root: an ordinary capability can walk *down* a realm but not *up* past its root. Above all realm roots sits exactly one privileged node, the **realm registry**, whose children are the realms. Traversing into it needs the realm-authority capability, held by the installer, backup tooling, and factory reset, not by ordinary principals. So there is still one global thing, but it is an *authority* almost nobody holds, not a *namespace* everyone shares.

### Copying is a reference operation

Because bodies are content-addressed and a name is a reference to a hash, copying a file is creating a new reference to the same body. No bytes move, so a copy is O(1) regardless of file size. Directory nodes are content-addressed too (a directory's content is its list of `name -> hash` child entries), so a directory's hash transitively names its entire subtree. Copying a whole folder is therefore one reference to the existing tree root, not a walk that recreates a million nodes. A snapshot is the same operation over a realm root.

The store is a Merkle DAG, not a tree: the same immutable node can be referenced by many parents, so copies share nodes. This is safe because nodes are immutable, so two references can never interfere; modifying a copy never edits a shared node, it writes new nodes along the path from the change to the root (copy-on-write path-copy, O(depth)) and leaves every untouched subtree shared. The DAG is also acyclic *by construction*: a node's hash depends on its children's hashes, so a cycle would require a hash to be known before it is computed. That means traversal always terminates and needs no cycle detection, unlike symlinks. The one place sharing is visible is aggregate queries: "how big is this folder" must walk with a visited-set so a shared body is counted once, and storage accounting can answer honestly that ten copies of a 100GB folder cost about 100GB, not a terabyte.

Bytes only actually move when a copy must live on different physical storage (cross-device, or a realm backed by another drive), which is the online-relocation path, and even then content-addressing dedups it: a body the target already holds transfers for free.

### Files, copies, and aliases

A content hash names fixed bytes, but a file that changes needs an identity that survives its edits, so a file is two levels: a small mutable **cell** with stable identity that points at its current content hash, over the immutable content the cell points at. That split makes the three operations users expect fall out cleanly:

- **Copy** is a new cell pointing at the same content. Two independent files sharing a body until one is written, at which point copy-on-write gives that one a new content hash and leaves the other alone.
- **Alias** (the hardlink) is two names for the *same* cell. A write goes through the shared cell, so both names see it. There is no original and no owner: the cell lives while any name references it and is freed when the last one goes. "If the original is deleted the other takes over" is just refcounting on the cell, and the reason that feels like a fiction is that it is one.
- **Symlink** is a name that points at another name or path, and dangles if its target is renamed or removed. It is the weaker shortcut, for when you want a pointer that follows a location rather than a thing.

Copy and alias differ on purpose: copy duplicates identity, alias shares it. Snapshots and dedup are unaffected, because the content is still content-addressed and a snapshot just freezes every cell's current content pointer into an immutable point-in-time tree. Whether the mutable-cell level exists at all is a real design choice: a purely path-identified store (Git-like) gives copies but not true aliases.

### Cross-realm access and the honest cost

Cross-realm sharing is a capability-gated edge, not a global path. When an app needs a user file, something mints a capability into the user realm for that one object: that is the file picker as an authority mint ([[human_permission_ux]]). The distinction to keep explicit, which iOS and Android both blur, is app-private storage (the app realm) versus user-facing documents (the user realm): a file an app saves "for the user" lands in the user realm via a granted capability, not buried in the app's container.

The tradeoff, stated plainly: a global root is a composability win, realms are an isolation win, and you cannot fully have both. A bare path is no longer a universal address, so sharing across realms costs a capability plus a qualified name instead of passing a string, and a persisted reference must record realm-plus-path, not a bare path. A global object store still exists underneath, so realms partition *naming and authority*, not storage. And "see everything" for admin, backup, or search needs a meta-capability instead of being free.

### Storage is relocatable

Because names resolve through the object graph and nothing addresses a device offset, the physical backing of any object or subtree can move at runtime while everything keeps running. Right-clicking a folder and choosing "move to another drive" is an ordinary online operation, and it works on any subtree, up to and including the entire system realm, with the system still live.

This falls out of the model rather than being a feature bolted on. Realms already decouple a name from its storage. Content-addressed bodies are identified by hash, so a body is location-independent and moving it re-places bytes without renaming anything. The single-level store ([[memory_and_persistence]]) already treats placement as a movable property, so "another drive" is just another placement target on the tier continuum. And because the store is a database, relocation is a routine online migration: copy the body to the new backing, switch the reference, drop the old copy, all committed as a transaction ([[transactions_and_consistency]]) so a power loss mid-move leaves either the old backing or the new one, never a corrupt half.

Omega is what lets the running system absorb this. A handle is a capability over an object, resolved through the graph, not a raw pointer to a disk location, so relocating the backing does not invalidate it; the runtime re-points the handle atomically. Because the OS holds the handle and lease graph, it knows exactly which components have the object open and can quiesce or re-point them, the same machinery hot swap uses to reach a safe point ([[updates_and_hot_swap]]).

The friction is that an app actively writing a file is mutating the bytes being moved. A cooperative component can be asked to quiesce and flush. The general case uses live-migration technique: copy the bulk while writes continue, track the delta from the log (change data capture, [[transactions_and_consistency]]), then take a brief cutover freeze to apply the final delta and switch the reference. A foreign or uncooperative app can delay the cutover but cannot corrupt the move, because the OS owns the indirection; the worst it can do is hold the file open, the familiar "file in use."

The real limit is large, hot files. A continuously appended high-bandwidth file, such as an active video capture, resists live migration for the same reason a write-heavy VM does: if the write rate exceeds the copy bandwidth, the delta never converges. The honest options are to wait for the writer to rotate or finish, force a cutover the writer must tolerate, or move only the cold prefix and leave the hot tail in place until it cools. The OS should pick and say which, rather than pretend the database makes it free.

This works only because no component's address space is ever pinned to a storage location, which is why Cathedral has no memory-mapped files. An `mmap` hands an app raw pointers into specific physical pages, so the backing cannot move without invalidating them. The legitimate uses of `mmap` are served by location-independent primitives instead: paged access to large persistent data is the single-level store ([[memory_and_persistence]]), a zero-copy read is a borrowed view over an object's pages with the runtime still owning placement, and shared-memory IPC is the shared-region primitive ([[ipc_and_service_invocation]]). The app holds an object reference or a borrowed view; the runtime owns where the bytes live.

### Placement and durability classes

Where an object's bytes live, and how many copies exist, is a **placement and durability class** tagged on a subtree, separate from which realm names it. Realms are namespace and authority; placement is physical policy; the two are orthogonal, so changing how something is stored never means moving it between realms. The class spans a simple range:

- **Default.** The runtime chooses the tier and location, the single-level store's normal behavior ([[memory_and_persistence]]).
- **Pinned.** One copy, on a named drive. A games folder on a specific disk is a subtree pinned there. This is an explicit re-coupling of logical to physical that the user opted into, while the default stays decoupled.
- **Mirrored.** N copies across a set of drives. Writes go to a primary and propagate to the replicas; reads can serve from the nearest. Because bodies are content-addressed, a replica that already holds a hash costs nothing to copy.

Mirroring is easy exactly when there is no write contention. The immutable system realm replicates trivially: copy once, newest version wins. Low-rate, effectively single-writer data such as OS configuration replicates on commit just fine. The hard case is high-rate multi-master data, the same user document edited on two machines at once, which needs conflict resolution ([[distributed_boundary]]) and is the one thing you keep out of the cheap synchronous class. The boundary is contention, not mutability.

This subsumes the boot story: tag the system realm and the boot-critical config as mirrored across the enrolled drives, and every drive becomes a self-contained boot drive, with the recovery image as the degenerate one-copy case ([[boot_and_trust_chain]]). Changing a class, starting or stopping mirroring or moving a pin, is a managed cutover, the same online-migration machinery as relocation above, not a flag flip.

### Virtual realms: sandboxing and containers

A realm root is an *interface* (resolve a name, list children, read or write an object), and a component only ever talks to the root capability it holds. It has no global root to escape to, so the world a component sees is entirely determined by what sits behind the root it was handed, and that can be anything that implements the interface:

- **Attenuated.** The real `system://` root, narrowed to read-only (a `Realm::ReadOnly` domain). The component sees the truth and cannot write.
- **A view.** A root that delegates reads to a real realm but hides or substitutes subtrees. Directories are already views, so a realm root can be a masking one.
- **A copy-on-write overlay.** Reads fall through to a base realm; writes land in a private per-component layer. The component edits "system files" and is really writing its own overlay, the base untouched and, because bodies are content-addressed, shared with no copy. That is `overlayfs`, for free.
- **Fully synthetic.** The root is backed by a provider that fabricates the tree: a made-up `system://`, a made-up `user://` with apps and files that do not exist. The provider answers however it likes, and the component believes it is running on a whole OS.

The component cannot tell which it was given, because resolving names against its root is the only thing it can do.

This is how Cathedral subsumes Docker. A container is a stack of Linux namespaces (mount, pid, net, user) plus cgroups and seccomp, all bolted on to carve a private view out of a system that has a global everything. Cathedral has no global anything, so each piece is a primitive that already exists:

- the filesystem view is the root you hand it (above);
- the visible component set ("what is running") is capability-scoped already ([[observability_and_introspection]]), so a process listing inside shows a fabricated or empty set;
- the network is flow authorizations to specific peers ([[networking]]); a private or synthetic network is a provider answering them;
- resource limits are budgets ([[scheduler_and_resources]]);
- syscall filtering is the effect ceiling.

So a "container" is an app handed synthetic or attenuated roots and a scoped capability set. There is no container runtime and no image format: the sandbox is the default you get by not handing over the real roots, and it **nests**, because a sandboxed component can hand its own children further-attenuated roots. It is the same machinery as the deterministic test world ([[testing_and_simulation]]) and live migration: a container, a hostile simulator (lie about time with a virtual clock, lie about the network, inject faults), and a snapshot-and-move are all "the component's world is the capabilities and roots you gave it."

The illusion can go all the way to root. An app can be handed not just a synthetic root but the *authority* over that realm, the same kind of capability the realm registry holds at the top of the real tree. Then the app is root of its world: it can mint capabilities, grant and attenuate them, revoke them, spawn privileged children, and carve sub-realms, with real operations scoped to synthetic objects. It is running its own miniature Cathedral and can sandbox its own children exactly as it was sandboxed. One invariant keeps this safe: **a realm-authority can only mint capabilities over objects within its own realm.** The app can mint endlessly, but every minted capability resolves through its realm's graph and names a synthetic object, never a real one; its only bridges outward are the few real capabilities it was granted.

Authority nests downward, and the host owns all of it. The realm registry is the root of the whole tree, so every realm is a descendant the host holds ancestor authority over, and a parent realm's authority dominates every child's. The sandboxed app is sovereign inside its realm while the host reaches into it from above, because the host owns the realm the app's realm lives in. Two things stay real no matter how root the app feels: its children draw on the app's actual resource budget ([[scheduler_and_resources]]), not the unlimited one it imagines granting them, and the whole fabricated world, including every capability the app minted, is one attributable subtree the host can read ([[observability_and_introspection]], [[audit_compliance_provenance]]). Sovereign inside, contained outside.

The limit is honest. Fooling an app into believing it runs on *Cathedral* is free, because every surface it touches is already providable. Fooling an app that expects a *specific other OS* (a Linux binary wanting `/proc`, `fork`, signals, `mmap`, ioctls) means emulating that ABI, which is the compatibility box ([[compatibility_and_legacy]]), not the realm. The realm gives the namespacing for nothing; the foreign syscall surface is the work.

## Concerns & Design Space

- **Virtual realms.** A realm root is an interface, so a component can be handed an attenuated, view, overlay, or fully synthetic root and cannot tell the difference. This is the native form of containers and sandboxes ([[security_policy_and_sandboxing]]): overlays fall out of content-addressing, and sandboxes nest for free.
- **Realm-authority delegation.** An app can hold the *authority* over its synthetic realm, becoming root of it (mint, grant, revoke, sub-realm), confined by the rule that a realm-authority mints only within its own realm. Authority nests downward, and the host owns every realm through ancestor authority ([[capability_model]], [[identity_and_principals]]).
- **Per-principal resolution.** Realm names (`system:`, `user:`) bind to roots in a private per-principal environment set at spawn ([[component_model]]), so the same qualified name resolves to different objects for different principals. Rebinding a name to a different root is how a sandbox is built, and the host owns every binding because it owns the tree.
- **Records and views.** Files as typed records; directories as indexes/views; a query layer ([[observability_and_introspection]]) over metadata rather than `find`/`stat` loops. Search/indexing is a first-class service, scoped to the realms you hold.
- **The transaction log as the source of truth.** Journaling is exposed *as an API*: change data capture is reading the log forward from an offset. Snapshots are log positions; rollback is replay to a position ([[transactions_and_consistency]]). The log is likely per-realm.
- **Durable subscriptions.** File watching becomes a resumable cursor over the log: a subscriber holds an offset, survives reboot, and is guaranteed every change in order with no overflow drops.
- **Atomic multi-object change.** A rename, a config write, and a content update commit as one transaction; the primitive lives in [[transactions_and_consistency]] and this chapter is its first heavy consumer.
- **Content addressing & dedup.** Object bodies keyed by content hash give cheap dedup, integrity, and snapshot sharing across realms; names are mutable references into it.
- **Realm membership by ancestry.** Membership is the root the parent chain ends at, not a per-file tag; the optional cached `realm_root_id` is a denormalization, and a cross-realm move invalidates that cache over the moved subtree.
- **The system realm is swappable.** It wants to be immutable and content-addressed so the OS can version and roll it back independently of the user realm ([[versioned_state_and_migration]], [[updates_and_hot_swap]]); reinstall swaps the system realm and leaves the user realm untouched.
- **Online relocation.** Any subtree's physical backing can move between devices at runtime, because names resolve through the graph and bodies are content-addressed; the move is a transaction with a live-migration cutover for in-use files ([[transactions_and_consistency]], [[updates_and_hot_swap]]).
- **Placement & durability classes.** A subtree is tagged with where it lives and how many copies it keeps (default, pinned, mirrored), orthogonal to its realm. Mirroring is cheap only without write contention, and changing a class is a managed cutover ([[memory_and_persistence]], [[distributed_boundary]]). Mirroring the system realm across drives makes every drive a valid boot drive ([[boot_and_trust_chain]]).
- **Mutable file identity.** A file is a mutable cell over immutable content, which yields copies (a new cell), aliases/hardlinks (a shared cell with no owner), and symlinks (a name pointing at a path). Whether the cell level exists is a real design choice with snapshot and dedup implications.
- **No memory-mapped files.** Nothing pins a component's address space to a storage location, which is what makes relocation and tiering possible. `mmap`'s uses are covered by the single-level store, borrowed object views, and the shared-region IPC primitive ([[memory_and_persistence]], [[ipc_and_service_invocation]]).
- **Per-object capabilities.** A handle to an object is the authority over it; a directory view is an attenuated capability ([[capability_model]]), and a realm root is just the top-most such capability.
- **Provenance.** Each record carries who/what/when wrote it ([[audit_compliance_provenance]]).
- **Schema evolution.** Records have typed, versioned shapes that migrate ([[versioned_state_and_migration]]).
- **The on-disk format is `wire data`.** Durable, source-of-truth metadata (directory nodes, file records, log entries, the superblock) is a versioned `wire data` schema with stable field numbers, so it decodes across OS versions. Storage is the hardest compatibility case, because a node may have been written by a version that now runs nowhere: you never break decode of an old node and you keep a total migration chain ([[versioned_state_and_migration]]). The superblock is the most frozen of all, a tiny self-describing header (format version, root hash, log head) the boot chain reads before anything else is reachable. Derived structures like the `hash -> location` index are the exception: version-tagged but rebuildable, discarded and regenerated on a mismatch rather than migrated.
- **Offline/online sync & conflict resolution.** Causal history makes merge and conflict detection a property of the log.

## Key Questions

- What is the unit of a "record" (a whole file, a sub-object, or a typed fragment), and how does that interact with huge opaque blobs (video, disk images) that resist structure?
- What is the right realm granularity: one realm per app, or coarser, and who is allowed to mint a realm before it becomes fragmentation ([[governance_and_extension_boundaries]])?
- Is the log global, per-realm, or per-subtree, and how is ordering defined across objects that commit together vs. independently?
- A shared object reachable from two realms has no single home; is its "home realm" its creator, with others reaching it by a cross-realm edge, or is realm membership genuinely multi-valued?
- What is retained forever vs. compacted, and who pays for unbounded history?
- Is there a write rate above which live relocation cannot converge, leaving a hot subtree pinned to its device until it cools, and how is that surfaced to the user who asked to move it?
- What does a synthetic or overlay realm cost at the resolution layer, and is a fully fabricated `system://` cheap enough to back a long-running sandbox, or only a short-lived test world?

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
- How is an open handle re-pointed across a relocation atomically, with no window where a read sees neither the old backing nor the new one?

## Related
- [[capability_model]] — per-object capabilities, realm roots, and attenuated directory views.
- [[naming_and_discovery]] — realm-qualified names and resolution as a security primitive.
- [[human_permission_ux]] — the file picker as the cross-realm capability mint.
- [[multi_user_and_org_control]] — tenants as realms.
- [[transactions_and_consistency]] — the atomic-commit primitive this chapter consumes.
- [[versioned_state_and_migration]] — schema evolution and the swappable system realm.
- [[memory_and_persistence]] — the single-level store, tiered placement, and why there are no memory-mapped files.
- [[security_policy_and_sandboxing]] — virtual realms as the native container/sandbox.
- [[compatibility_and_legacy]] — synthetic realms vs. the foreign-ABI box.
- [[audit_compliance_provenance]] — provenance recorded per change.
