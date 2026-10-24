# Owner questions

Only unresolved owner-level Cathedral semantic, compatibility, architecture,
or trust decisions belong here. Settled decisions live in the
[specification](wiki/spec/README.md); implementation work and deliberately
deferred research do not.

Before adding a question:

1. Name the concrete product or implementation requirement that is blocked.
2. Cite the specification owners and machine-readable contracts checked and
   found silent.
3. Separate the owner choice from ordinary engineering or extraction work.
4. State the viable choices, their invariant and compatibility consequences,
   and a recommended answer.

A missing specification page is not by itself an owner question. Neither is an
implementation difficulty. Record design conflicts and oddities as they are
encountered when they expose an owner choice affecting an active requirement
or architecture boundary. Ordinary documentation corrections can be fixed
directly; recommendations here remain proposals until accepted.

Questions are a mutable decision queue, not stable contract identities. Code,
tests, and settled documentation must cite the specification clause produced by
the answer rather than an owner-question number. Resolving a question updates
the specification and affected source contracts in the same change, then
removes the question. Git retains the discussion history.

Existing `Key Questions` and `Open Questions` in design chapters are design
inventory. They are not automatically owner questions under this narrower
standard.

## Open questions

### DBFS deduplication domains and erasure

**Requirement and tension.** The filesystem design combines cross-realm content
deduplication, per-subtree sealing and a `wipe` retention mode. A shared backing
cannot be physically erased for one realm while another still requires it;
deduplication success, timing and accounting can also reveal content equality.
This blocks a production promise of global deduplication or secure erasure.

**Contracts checked.** The [filesystem design](wiki/design/part_4_storage/00_filesystem_as_database.md)
and [memory/persistence design](wiki/design/part_2_components/02_memory_and_persistence.md)
describe the intent; the [specification index](wiki/spec/README.md) and
[object protocol](source-rs/platform/contracts/storage.rs) define no encryption domain,
physical sponsorship, deduplication disclosure or erasure contract.

**Owner choice.** Deduplicate only within an encryption/trust/accounting domain,
or allow broader sharing with a specified confidentiality and key-lifetime model.
Define whether `wipe` means withdrawing this principal's access, destroying an
exclusive encryption key, or proving all managed copies unusable. These promises
are different, especially with replicas, snapshots and external copies.

**Recommendation (unaccepted).** Begin within one explicitly authorized domain;
keep hash existence and placement private. Do not promise global sharing or
immediate physical erasure. Charge retained backing to an identified sponsor and
define key/reference lifetime before enabling cross-domain deduplication.

### DBFS history and durable subscriber limits

**Requirement and conflict.** The filesystem design wants watchers that never
miss events and pressure-thinned history. An indefinitely offline subscriber
cannot have unlimited replay on bounded storage without eventually blocking
writes or increasing its funded allowance. This blocks a durable subscription
and retained-version API, not the current finite host experiment.

**Contracts checked.** The [filesystem](wiki/design/part_4_storage/00_filesystem_as_database.md)
and [transaction](wiki/design/part_4_storage/01_transactions_and_consistency.md)
chapters describe durable observation, but the
[specification index](wiki/spec/README.md) and
[Rust object protocol](source-rs/platform/contracts/storage.rs) have no cursor, retention,
expiry or subscriber-sponsorship contract.

**New evidence.** The [host engine stress lab](tools/storage-lab/engines/stress/README.md#reader-pressure-and-reclamation)
shows a long-lived reader retaining engine backing despite deletion of old records.
Releasing it permits repeated replacement under the same cap. This supports
explicit pin accounting; it does not decide expiry, sponsorship or reader denial.

**Owner choice.** Reserve replay/history storage and reject or block further
writes when exhausted, or permit bounded retention and require explicit cursor
expiry/resynchronization. Active-reader pins and explicitly retained versions
need a separate promise; silently removing backing from a live reader is invalid.

**Recommendation (unaccepted).** Offer bounded funded retention, report expiry
explicitly and support authorized resnapshot plus replay. Require explicit funded
pins for stronger history guarantees. Publish change records atomically with
state and specify duplicate delivery and revocation independently of CRDTs.

### DBFS isolation for cross-record invariants

**Requirement and tension.** The transaction design proposes snapshot reads and
first-committer-wins write conflicts while also promising transactional invariants
over several records. Disjoint writes based on a shared predicate can violate an
invariant under snapshot isolation even when neither write conflicts directly.
This blocks selecting the general transaction contract and its conflict indexes.

**Contracts checked.** The [transaction chapter](wiki/design/part_4_storage/01_transactions_and_consistency.md)
has no accepted read-set/predicate-conflict rule. The
[specification index](wiki/spec/README.md) leaves storage unspecified, and the
[Rust object store](source-rs/platform/libraries/database-objects/README.md) uses
a per-object expected revision rather than general snapshot transactions or queries.

The [host publication experiment](tools/storage-lab/engines/stress/README.md#preparation-and-publication)
now checks expected revisions under the engine writer, including whole-batch
rejection. It protects those explicit object revisions; predicates, independently
read objects and the general application isolation promise remain unspecified.

**Owner choice.** Require serializable transactions for general multi-record
operations, or expose weaker isolation with explicit invariant guards/stronger
transaction modes. The former requires conflict tracking or serialization; the
latter makes application obligations and retry behavior part of the public API.

**Recommendation (unaccepted).** Target serializable semantics for general
metadata transactions, initially allowing coarse serialization in the lab.
Specify bounded transaction sizes and retry outcomes before optimizing concurrency.
Do not equate copy-on-write, snapshot reads and serializability.

### Recursive cursor presentation and attribution

**Settled boundary.** The [presentation contract](wiki/spec/human_surface/rendering.md#cursor-presentations)
now selects per-pointer hover/capture targeting, recursive cursor preferences
including no-cursor, and owner-controlled restoration when the target changes or
dies. Hiding is separate from capture, confinement and relative-motion lock.
This is not keyboard-focus-based and does not require another registered focus group.

**Remaining requirement.** The design proposes trustworthy attribution for agents
and remote participants. It has not specified which indicators are mandatory or
must survive a child renderer that hides its ordinary cursor.

**Contracts checked.** The presentation contract permits bounded cursor surfaces
and scoped suppression; the [desktop contract](wiki/spec/human_surface/compositor_and_seat.md)
protects outer recovery, not every cursor indicator. The
[pointer proposal](wiki/proposals/0001_input_and_pointer_leases.md) retains
attribution and hidden-target presentation alternatives.

**Owner choice.** Define any mandatory agent/remote activity attribution and
protected fallback separately from ordinary cursor preference. Decide whether
hidden-target activity needs an owner indicator, optional preview, or no mandated
visual, and which identities the observer is authorized to see.

**Recommendation (unaccepted).** Keep ordinary cursor suppression local while
allowing separately authorized owner indicators. Do not label every decorative
cursor as authenticated or expose hidden participant identities by default.

### Exclusive audio and the reserved system channel

**Requirement and tension.** The [audio design](wiki/design/part_6_human_surface/06_audio.md)
offers exclusive device handoff that silences other streams, while promising a
reserved channel for alarms, accessibility and system sounds that apps cannot
suppress. On an output with no independently enforced mixing path, direct client
ownership cannot provide both guarantees at once.

**Contracts checked.** The [specification index](wiki/spec/README.md) contains no
accepted audio contract. The [source contracts charter](source/contracts/CHARTER.md)
defines no audio handoff or preemption semantics. The Rust lab has no audio
driver, graph or device lease implementing either promise.

**Owner choice.** Keep platform mixing for outputs that must carry the reserved
channel, allow an explicit policy exception during exclusive playback, or require
an independent enforceable path for reserved sound. Reclaiming a device on demand
is another possible policy, but requires a specified interruption bound and is
not uninterrupted reserved-channel availability.

**Recommendation (unaccepted).** Preserve platform mixing where the reserved
channel is required. Treat exclusive handoff as conditional on a defined policy
exception or an independently enforced output path. Measure reclaim behavior
before promising a bound. The [audio notes](wiki/drafts/audio_graph_notes.md)
leave this decision open until the device and interaction requirements are known.

### Kernel mechanisms versus platform orchestration

**Requirement and tension.** The kernel/platform split needs a defensible trust
boundary as the boot lab grows. The
[kernel charter](source/kernel/core/CHARTER.md) and
[TCB inventory](wiki/architecture/tcb.md) assign hot-swap, transaction
coordination, trusted loading and attestation to a kernel described as small
enough to audit in full. Those assignments do not distinguish enforcement
primitives from service orchestration. This is an unresolved scope decision,
not evidence that those features are impossible or that a particular line
count is required. The
[kernel architecture design](wiki/design/part_5_lifecycle/04_kernel_architecture.md)
also describes a narrower privileged substrate and restartable services, so the
charter and inventory need an explicit reconciliation with that direction.

**Contracts checked.** ADR 0002 establishes placement and dependency direction,
but does not decide this split. The specification index lists component lifecycle
and admission as unwritten subjects; its current boot and root-extent contracts
do not define these service boundaries. The
[contracts charter](source/contracts/CHARTER.md) names future interfaces without
settling their enforcement/orchestration split. The Rust lab now places its
private-object engine and recovery in a platform service over a boot-granted
PIO transport. That experiment covers disk-only atomic replacement; it does
not define a transaction that atomically changes persistent objects and kernel
authority, nor recovery of such a cross-boundary change. The broader transaction
coordination assignment remains unresolved.

**Owner choice.** Keep the listed subsystems wholly in the trusted kernel, or
place orchestration in platform services over narrowly defined kernel
mechanisms. The former expands the kernel audit surface. The latter requires
explicit authority, atomicity and failure/recovery contracts across the boundary;
moving code alone neither isolates it nor removes it from the TCB.

**Recommendation (unaccepted).** Default to platform orchestration, retaining
kernel mechanisms wherever required to enforce system invariants. Decide each
subsystem's boundary before implementing it, and update its specification,
contracts, charter and TCB entry together. Track kernel size and total trusted
code separately; do not adopt a 12,000-line target as an architectural constraint.

### Validation scope for an optional PCID optimization

**Resolved boundary.** The owner accepted [hardware-separated programs](wiki/spec/execution/isolation.md)
on 2026-10-23 and removed PCC-based single-address-space isolation from the
current roadmap. This replaces the former default-isolation question. PCID does
not require merging program address spaces; its test coverage is a separate choice.

**Requirement and choice (deferred).** The lab evidence below reports that the
current emulator path cannot exercise the proposed PCID optimization, while WHPX
can. Before enabling such a path, decide whether hardware/hypervisor-only coverage
is acceptable with a retained non-PCID fallback, or require broader target coverage
first. This does not block current hardware-separated execution or coordination.

**Recommendation (unaccepted).** Leave the optimization disabled until there is a
concrete implementation and validation plan. Do not infer approval of a different
kernel page-table layout or weaker isolation from the accepted program boundary.

### Writable row spans and component admission

**Requirement and evidence.** Direct rendering into delegated spans of a common
back buffer could avoid intermediate pixel copies. The
[rendering investigation](source-rs/distribution/lab/rendering/README.md) compares
that with flattened references to separate leaf buffers. For its opaque workload,
flattening already removes copies through ancestors. Checked Rust row borrows
also remove the final composition copy, but the QEMU probe demonstrates that an
arbitrary native store can cross an advertised row span within its writable page.
Crossing the mapped page boundary faults. These are different authority guarantees.

**Contracts checked.** The accepted [isolation contract](wiki/spec/execution/isolation.md)
requires hardware separation and excludes PCC-based shared-domain admission. The lab's
[page contract](source-rs/kernel/contracts/memory.rs) defines whole-page private writers
and sealed readers; it supplies no writable byte-range delegation or production
surface contract. The row wrapper relies on cooperating safe Rust and is not a
proof checker or native-code sandbox. No Omega admission guarantee is implemented
by this experiment.

**Owner choice.** Which admitted writers may share a writable protection domain,
and what enforces exact spans and eventual completion? Page-isolated native
writers, verified/sandboxed writers and trusted in-process components need not
use the same rendering path. A production contract must specify whether failure
can leave a private partial frame, modify visible scanout, or delay other writers,
and when a span can safely be reassigned after cancellation.

**Recommendation (unaccepted).** Keep hierarchy metadata separate from pixel
storage. Continue measuring flattened page-isolated leaves and checked direct
rows as distinct paths. Require an explicit admission and failure model before
promising exact sub-page containment or leasing visible scanout to independent
writers. This does not select owned surfaces; direct leases must respect the accepted
hardware boundary and cannot rely on the removed PCC-based SAS proposal.

The [rendering proposal](wiki/proposals/0000_rendering_and_composition.md)
keeps row borrows as a possible renderer implementation detail and evaluates
bounded scene submission as a separate route. It does not grant direct target
access to processes merely because Cathedral ships them.

### Platform drawing and text placement

**Requirement and tension.** A common renderer could execute bounded paths and
glyph runs from nested Matrices without requiring each app or ancestor to create
an intermediate image. The [media chapter](wiki/design/part_6_human_surface/02_media_and_graphics.md#text-and-fonts)
instead places all text rasterization in app libraries and states that the
compositor never sees a glyph. Its font-parser containment argument depends on
that placement. This blocks choosing the production drawing boundary, not further
measurement of either route.

**Contracts checked.** The [presentation contract](wiki/spec/human_surface/rendering.md)
settles placement, demand and backend boundaries but leaves drawing/text placement open.
[Source contracts](source/contracts/CHARTER.md) contain no accepted glyph or scene
protocol. The [Rust display contract](source-rs/platform/contracts/display/display.rs) has fixed
ASCII text, rectangle and sealed-pixel requests, all experimental. It does not
establish general font parsing, shaping or retained glyph semantics.

**Owner choice.** Keep the platform limited to completed images with all text/path
rasterization in app libraries, or also offer bounded platform drawing commands.
The latter lets the platform combine ordinary UI drawing but expands its rendering
and validation responsibility. Shaping/layout can remain reusable library work in
either choice. A trusted renderer parsing arbitrary font/image files has a larger
trust boundary than one consuming prepared, validated resources.

**Recommendation (unaccepted).** Evaluate optional platform drawing alongside
completed-image presentation, as described in the
[scene proposal](wiki/proposals/0000_rendering_and_composition/scene_submission.md).
Keep resource preparation and final target writes separable. Specify which font
and path interpretation enters the renderer's trust boundary before accepting
that route. Glyph drawing must not implicitly grant accessibility or text capture.

### Render-resource residency and sponsor liability

**Settled boundary.** The [resource contract](wiki/spec/human_surface/rendering_resources.md)
accepts leaf-funded local backing, authorized image versions, destination-side
uploads/conversions and safe deferred reclamation. Confirmed leaf death retires
live registrations, not memory still in use or independently held copies.
Accounting survives producer death until actual reclamation. Remote silence is
not proof of death. Registration alone is no blanket permanent-residency promise.

**Remaining requirement.** Derived caches and remote destinations need explicit
funding and missing-resource behavior before promising production availability.
Local source funding does not fund unlimited destination storage or descriptors.

**Contracts checked.** The accepted resource and presentation contracts leave
cache residency, exact sponsor selection and outage/miss presentation policy
open. The [Rust page contract](source-rs/kernel/contracts/memory.rs) exercises
fixed local grants and pinned readers, not a general remote resource service.

**Owner choice.** Select residency guarantees for particular accepted acquisitions,
cache eviction/reacquisition policy and the paying account for derived or remote
storage. Choose whether missing content retains authorized last pixels, shows a
placeholder or removes the contribution. These are not choices about whether
live readers may access reclaimed memory; that is already disallowed.

**Recommendation (unaccepted).** Negotiate bounded acquisitions separately from
registrations, account for derived/destination storage explicitly, and select a
miss/outage policy per service session. Keep source-independent copied resources
subject to their own authority and lifetime. The
[resource proposal](wiki/proposals/0000_rendering_and_composition/resource_custody.md)
retains the alternatives; no current remote/GPU implementation is implied.

### Endpoint transport and revocation of pending IPC

**Requirement and tension.** Isolated services need a transport and a defined
outcome when their authority is revoked during a wait. The
[IPC design](wiki/design/part_3_communication/00_ipc_and_service_invocation.md)
names shared regions as the one IPC primitive, with queues and RPC in libraries,
but its endpoint section also describes kernel creation and delivery of opaque
messages across hardware boundaries. It does not settle whether those are two
transports or descriptions of one shared-region protocol. The
[capability lifecycle](wiki/design/part_1_authority/01_capability_lifecycle.md)
says already-redeemed operations run to completion and revocation is discovered
on next use. An empty blocking receive may have no completion unless the contract
defines when redemption commits it and how cancellation or peer death intervenes.

**Contracts checked.** The [specification index](wiki/spec/README.md) assigns
authority semantics to the unwritten `spec/authority/capabilities.md`.
The accepted [coordination and IPC contract](wiki/spec/communication/ipc.md)
(2026-10-23) now establishes scoped introductions and protocol-controlled shared
memory as supported shipped operations. It does not select an exclusive transport
or settle pending-delivery transport outcomes. The subsequent owner clarification
accepts that coordination only controls introductions: established service
permission and cancellation belong to application protocols. The first participant
gains no implicit authority over peer mappings or independent rights, and closing
communication cannot silently invalidate live safe-language views. That ownership
boundary is resolved; exact transport closure and commit rules remain open. The
[Rust syscall constants](source-rs/kernel/contracts/user.rs) and
[transport model](source-rs/kernel/core/ipc/ipc.rs) explicitly define only lab
behavior. No accepted contract settles transport selection or pending-operation
revocation. This blocks promoting the experiment to a stable service interface,
not continued laboratory work.

**Owner choice.** Either require shared-region data movement with kernel endpoints
limited to authority and notification, or retain copied-message delivery as an
additional supported transport with explicit limits and equivalent protocol
semantics. Separately, define whether a parked receive has already redeemed
authority: allowing it to finish preserves the current lifecycle wording but
permits post-revocation delivery; rechecking at delivery cancels the wait but
requires distinguishing admission to a wait from committed message delivery.
Queued messages also need an explicit ownership/commit point so revocation
cannot silently change their promised outcome.

**Recommendation (unaccepted).** Treat copied queues as lab scaffolding until
the transport boundary is specified. Define separate wait, delivery and
completion points; permit cancellation before delivery commits, without
retroactively undoing completed delivery. Reconcile that rule with the lifecycle
chapter before freezing an API. The current Rust experiment rechecks pending
receives, returns `REVOKED`, and discards undelivered queued bytes on revocation;
those are explicit experimental choices, not an accepted production policy.

**Additional lab evidence.** The [page-object contract](source-rs/kernel/contracts/memory.rs)
now supplements copied control messages with sealed shared buffers. Sealing
removes producer write permission; accepting a read-only lease pins backing even
if the producer dies. Unaccepted offers disappear on producer death, and peer
replacement cannot inherit them. A live accepted reader is never remotely
unmapped; release or reader death ends its lease. QEMU exercises this lifetime,
hardware permissions, resource bounds and full reclamation. The visible display
demo uses this path for pixels. The accepted coordination boundary now rules out
implicit removal of those views by a group's first participant or by withdrawing
discovery access. Any separately authorized mapping teardown must still preserve
live safe-language borrows. Neither this experiment nor its Rust wrappers settles
general capability revocation or whether copied control messages remain a
supported transport.

**Further IPC evidence.** The [ordinary channel lab](source-rs/distribution/lab/channels/README.md)
now implements explicit moves of dynamic endpoints. Successful send removes the
sender's grant and reserves a hidden recipient grant; receive publishes it only
after checked copyout validation. Sender death does not undo the move, while
receiver death or bootstrap revocation discards an unaccepted attachment. This
separates queue commitment from recipient acceptance and from service completion.
It does not define a parent-revocable delegation tree. Production semantics still
need to choose the cancellation/commit point and who sponsors in-flight and
transferred endpoint storage. The fixed lab reservations are evidence, not an accepted
hierarchical resource-accounting contract.

**Rust sponsorship experiment (not resolution).** The
[channel lab](source-rs/distribution/lab/channels/README.md#bounds-and-remaining-work)
now reserves backed channel capacity for each boot/launch account. Charges survive
creator death and slot replacement; endpoint moves change custody without moving
funding. Final endpoint closure refunds the original account. QEMU retains channels
across eight creator deaths, rejects further creation, proves a sibling can fill
its reservation, and restores capacity after final closure. This closes the lab's
restart loophole but allows an accepted receiver to pin a replacement's allowance.
Production policy still needs to choose persistent sponsorship, explicit funded
acceptance/transfer, or a defined teardown protocol. It must not silently erase
charges or revoke accepted readers as a side effect of accounting. Static launch
reservations do not settle hierarchical subdivision, CPU budgets or timing isolation.

### A parent exercising a child's binding to warm the chain's proofs

**Requirement and tension.** A storage request carries two scope stamps and no
more. The matrix that would add a third proves instead, once per binding and
pair of scopes, that the innermost scope lies within the other two: a HEAD on
that scope stamped with both, answered by the provider and memoized under the
binding until a revoke, a regrant, an upstream revision change or provider
retirement discards it
([storage.rs](source-rs/platform/libraries/matrix/world/storage/storage.rs)).
Each proof is raised by the first request that needs it, so a depth-4 matrix's
first request waits for four such chains one after another: 83 ms of its
315 ms startup under TCG with traces, roughly 20 to 30 ms without, and within
the trace line's own noise on hardware (TASKS_RUST.md, the hop on hardware).
The only way to take them off the startup's path is for the granting matrix to
send a request on the child's binding at grant time, before the child asks
anything, so the ancestors prove the new scope while the child's program loads.

**Contracts checked.** The [matrix charter](source-rs/distribution/matrix/CHARTER.md)
has a parent act on a child's storage only to grant, revoke and forward; a
parent's own reads use its own binding
([pending.rs](source-rs/distribution/matrix/service/parent/storage/pending.rs)).
The scope machinery folds a request only after a confirmed proof under the
exact binding; it does not say who may raise one. Nothing forbids a parent
reading through a child's binding and nothing permits it: today no request
travels under a binding unless the bound child sent it.

**Owner choice.** May a matrix exercise a child's binding without the child
asking, for a read whose only effect is the proof caches along the chain? If
yes, the stock matrix warms every grant it makes with a HEAD on the granted
scope through the child's slot, and the chain's proofs are in place before the
child's first request. If no, the proofs stay lazy and the first request
through a new binding pays for them: a few hops on hardware, tens of
milliseconds under TCG.

**Recommendation (unaccepted).** Leave the proofs lazy. A request the child did
not send is a new kind of traffic on the trust path, seen by every ancestor as
the child's, for a saving the harness cannot resolve on hardware and that only
emulation makes visible. If the chain should be warm, the child's own first
request is the cleaner gesture, and it is what happens now.

**Lab evidence (2026-10-23).** Tried as an experiment: the granting matrix sent
a HEAD on the new scope through the child's binding right after the grant, as
its own exchange with nothing to answer, and the child's first request waited
behind it as every request waits behind the one exchange a matrix has in
flight. On the lab's single CPU the proofs then ran before the child's request
instead of beside it, plus the warm request itself: the depth-4 startup chain
measured 357 to 385 ms with eleven provider requests against 312 to 365 ms with
ten in alternating runs (`build/warm-*.log`), and no entry step improved. The
code is not in the tree. The choice above matters only on a host where the
chain could overlap the child's own start, and there the hops it would save
cost microseconds.
