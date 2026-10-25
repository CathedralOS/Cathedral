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
