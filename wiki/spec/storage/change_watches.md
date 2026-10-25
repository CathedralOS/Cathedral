# Object-rooted change watches and durable capture

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | None |
| Last reviewed | 2026-10-25 |

## Ownership and scope

The storage service owns watch registration, hierarchy matching, event production
and durable capture. The kernel supplies ordinary mapping, funding and
[notification](../execution/notifications.md) mechanisms, not filesystem event
queues, hierarchy traversal, summaries or audit history. These are service and
library contracts, not new Omega language primitives.

There are two delivery guarantees: live observation into caller-funded memory,
and optional durable capture coupled to storage commits. Neither promises
unlimited retention. Durable change records do not imply retained historical
file contents, read-access auditing or a general replication protocol.

## Object identity and authorized views

A watch binds to an authorized logical object instance, with an explicit recursive
or direct scope, not to a path string. Replacement at the old path does not retarget
it. Independent copies have independent watch identity even when their immutable
backing is shared. This requires no public persisted GUID scheme; it preserves the
[path and copy contract](realms_and_objects.md).

Every granted realm is a rooted view. Its `/` exposes no enclosing parent, global
root, owner or outside path. Moving the watched root itself therefore produces no
event in that watch solely for the outer movement. Parenting and names belong to
parent-child relationships, not an implicitly readable parent field on the child.
The service may keep private reverse links for matching without exposing them.

A recursive watch observes authorized descendant changes. Matching uses the
hierarchy before and after the committed mutation: a departure is removal from
the old view, an arrival is addition to the new view, and a watcher authorized for
both may see a move. An outside destination or source is not disclosed. Moving a
subtree is one logical hierarchy operation with subtree scope, not a synthetic
move for every descendant. A watch rooted on that subtree continues unchanged.

Observation rights do not grant mutation, backpressure or broader enumeration
rights. Revocation stops new disclosure and access to service-held history under
that grant; it cannot erase records already delivered. Object numbers, cursors,
buffer addresses and process IDs alone grant no authority.

## Committed events and initial enumeration

The full feed accounts for committed content, object-metadata and hierarchy-entry
changes within the active scope. Records identify affected objects/entries and
the authorized before/after relationship or change flags, with commit grouping
and stable event identities. Deleted-object records remain meaningful without
dereferencing deleted storage. Content change reports a change/revision, not a
byte diff or a promise to preserve previous bytes.

Several writes within one transaction may produce one committed change for an
object; separate commits remain separate changes. Failed attempts and ordinary
reads are not completed-change events. Ordering follows the relevant commit
scope, not a required machine-wide total order. Exposed cursors are scoped opaque
positions and must not reveal unrelated realms' activity through global counts.

Registration establishes a precise starting commit boundary. An authorized
consistent enumeration can be paired with a starting change position so an
indexer can enumerate and follow changes without a gap. The same requirement
applies when enumerating an arriving subtree. A consistent view is not a copy of
every file's bytes; retained enumeration state must be funded and kept valid
through its declared lifetime. An ordinary object reference does not freeze it.

Applications or libraries can summarize the full feed into per-object changes,
including combined content, metadata and hierarchy flags. Such a summary is a
lossy semantic view, not full history; it must not be advertised as all intervening
events. Lossless record compression is a different matter. The service does not
switch architectures when a second watcher arrives and need not implement summary
queries.

## Live delivery and bounded storage

Registrations bind the owner/incarnation, object scope and authority, a validated
shared-buffer grant, notification right, producer position and overflow policy.
The backing's validated size and queue layout determine delivery capacity; a
peer-written header cannot claim additional space. Producer and consumer writable
regions/rights are separated, and hostile counters, lengths and acknowledgements
are validated. Closing a watch retires or cancels in-flight writes before its
backing can be reclaimed.

Self-contained records are routed directly into caller-funded shared rings.
Ordinary watches do not require a central retained event arena or queues of handles
that secretly retain service-owned history. Interested watchers may receive copies
of the same metadata. Consumers own draining, summaries and any further storage.
The service still funds registration and bounded transaction staging/reservations.

Watch lookup may use object-indexed registrations and private ancestor links.
Mutation matching visits relevant old/new ancestry and delivers to matching
watchers; it need not scan every registration or expand a subtree move into all
descendants. Exact arenas, indexes and queue encodings are implementation choices.
Use the existing [shared-memory IPC boundary](../communication/ipc.md).

Default live overflow stops capture and exposes a persistent gap/overflow status;
it does not silently overwrite unread events. That status must remain observable
even when no event slot remains. Resuming after a gap requires explicit
resynchronization, not presenting the remaining records as complete history.
Notification is a wake hint, not evidence that records were consumed.

Lossless backpressure requires explicit authority to refuse or delay affected
mutations. Capacity is reserved before committing such mutations; observation
permission alone cannot stall other writers. No finite buffer promises an
unbounded unattended watch. Releasing a RAM slot, processing an event and
acknowledging durable retention are separate operations.

## Durable capture and recovery

Durable subscriptions or administrator-required audit policies additionally retain
committed events in service-managed persistent storage. The storage transaction
publishes the mutation, its required change-event information and the commit
decision together. Required capture capacity is reserved before commit. Aborted
transactions do not become completed-change events. A crash after commit but
before delivery must leave the events replayable.

This reuses the engine's recovery machinery. It may retain sufficiently descriptive
transaction records or store compact event records transactionally alongside
ordinary data. WAL is a recovery mechanism; the authorized committed change feed
is its consumer-facing contract. Baseline crash recovery is not switched off when
nobody watches. Optional extra event capture/retention is only required while a
watch or policy demands it.

Persistent history uses ordinary allocated storage, able to grow like other stored
data within its funded allowance and free-space limits. No fixed audit partition,
separate database or kernel-wide mega-log is required. Delivery-buffer pages fund
RAM delivery, not disk retention. The service must account for retained records,
recovery and reclamation headroom; a reader or subscription cannot silently retain
uncharged unlimited backing.

Registration/policy and its starting position are committed before success.
Registration, cancellation and changes to observation authority are ordered with
mutations so their capture/disclosure boundaries are unambiguous. Durable policy,
subscription state and acknowledged positions recover after service restart.
Collector disconnect is not cancellation. Redelivery is permitted, with stable
identities for deduplication; exactly-once application processing is not promised.

A collector persists what it needs before acknowledging. Acknowledgement releases
eligible replay obligations, but does not waive an administrator's mandated audit
retention period. A full delivery ring need not stop mutations while persistent
capture has reserved capacity. When guaranteed persistent capture cannot be
funded, covered mutations refuse or wait under the authorized policy; the service
must not silently turn auditing off or discard promised history. Explicit policy
retention limits and cancellation determine eligibility for reclamation.

Preparation and delivery may run concurrently. Commit ordering and capture
reservation remain coordinated; batching/group commit is permitted. A single
logical producer per ring is sufficient initially. Neither lock-free structures
nor a global serial application callback are required. Post-commit delivery cannot
substitute for atomic durable capture.

## Always-on audit policy

An authorized administrator can persist mandatory capture policy in protected
system-realm configuration. The storage service recovers and applies it before
admitting ordinary covered mutations. A missing or unreadable required policy
does not mean auditing is disabled. A collector may start later without a capture
gap; ordinary applications cannot cancel the policy. Provisioning policy before
issuing write grants is also valid, but earlier setup is outside that guarantee
unless separately covered.

Policy changes require the policy's control authority and are transactionally
recorded. Audit-record maintenance and delivery must not recursively audit their
own internal writes; authorized history deletion or policy changes remain explicit
administrative events. This guarantee covers mutations mediated by the storage
service under its trusted backing assumptions, not bypasses by a more privileged
raw-device writer or a compromised service.

## Implementation boundary

The [object storage service](../../../source-rs/platform/services/storage/README.md)
and [object library](../../../source-rs/platform/libraries/database-objects/README.md)
provide the starting transaction and hierarchy mechanisms, not this watch contract.
Existing shared-buffer/notification proofs likewise do not implement watches.
Remaining acceptance is tracked in the
[watch outcome](../../../TASKS_RUST.md#object-rooted-watches-and-durable-capture).

Wire spelling, queue layout, physical log segmentation and indexing remain
engineering work. [Explicit atomic batches](atomic_batches.md) settle revision
prerequisites and all-or-nothing mutations in one service transaction domain, not
general query-aware isolation. Aliases, cross-domain
deduplication and historical-content erasure are not settled here. Surface new
semantic or trust gaps through [owner questions](../../../OWNER_QUESTIONS.md), not
by silently weakening this contract.
