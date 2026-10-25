# Concurrent input and presentation

The stock matrix keeps its existing scoped interaction connection. It permits
one tagged input poll beside one asynchronous presentation exchange. No global
locator, new child capability, kernel queue size or public packet shape changes.

Follow [the service loop](../service/service.rs) into
[interaction.rs](../service/parent/interaction/interaction.rs), the sole upstream reply reader
while these exchanges are active. [transport.rs](transport.rs) distinguishes
tagged snapshots from canonical untagged presentation status. This concurrent
client uses poll IDs 1 through `u32::MAX - 1`, never reuses them and reserves the
high words used by signed status. The public serial poll protocol is unchanged.
Copied fixture streams ignore unmatched poll tags. Shared streams bind each
transport ID to its input or visual owner and require the corresponding application
reply. A mismatched tag or malformed status is fatal rather than uncertain success.

[polling.rs](../service/parent/polling.rs) applies input snapshots;
[presentation.rs](../service/parent/presentation.rs) retains the scene and advances
begin/update/commit/abort. Local input queries remain serviceable even from the
publishing child. Startup holds and authority-changing requests still wait.
A snapshot replacing the view or surface capacity is retained until publication
ends, then consumed before parking. Malformed input that invalidates a publishing
binding fails closed. A slow reply alone neither revokes input nor replays work.

Ordinary separate-stream composition uses
[the connection adapter](../service/parent/interaction/connection.rs): a persistent
`Pipeline<2>` owns the parent pair. One logical input request and one logical
visual request may be outstanding together. A response consumes its exact transport
ID; its packet must then match the owning logical operation. Synchronous operations
wait until both owners retire. Late application work remains live, while incomplete
attachment custody has a bounded deadline. Waits include both notification and
endpoint lifetime. Immutable image grants keep their existing ownership.

The service preselects pipelining for its five child interaction slots; storage
uses independent one-call connections. Admission retains a writer page, reader
page and notification allowance for each selected server end before lending the
rest to the child. Small fixtures fund both ends explicitly. Slot removal drops
shared custody before reclaiming the child's remaining connection resources.

Manually composed fixtures with combined interaction/storage streams, Omega leaf
interaction and Split interaction still use copied packets. Copied replies retain
the runtime's [bounded reply arena](../../../platform/libraries/user-runtime/server/replies.rs).
Composition selects this path; failed shared setup never falls back. Unpublished
mailbox refusal retains the operation for a later turn. Consumption notifications
can wake a pending peer, and an unsent request also uses a one-tick retry deadline.

`desktop_test.py --nested-desktop --interaction-delay --timeout 240` holds a real
presentation for five seconds and requires fresh keyboard input in every matrix
realm on the three-level focused chain before release, on fresh and cold boots.
It also exercises delayed polls/images, protected close, saved Notes
and Colors, and recursive reclamation. `run.py --ipc-test --release` exercises
full reply channels, ordering, stale incarnations and queue overflow directly.

This proves input delivery through nested matrix routers during asynchronous
presentation. An app blocked in its own synchronous drawing call still cannot
process that input until it returns. Launch activation now uses the same dispatcher
through [scene/scene.rs](../service/parent/scene/scene.rs); publication is a local
startup handoff. Chrome selection of existing apps and its cancellation restoration
also yield through that scene owner; follow
[workspace/selection.rs](../service/children/workspace/selection.rs). Root
physical-source dispatch, image calls, lifecycle-driven fallback selection and
launch teardown/compensation retain synchronous work. An atomic scene
still serializes visual commits. This is not a claim of end-to-end latency bounds
or that sibling pixels can commit into another app's open scene transaction.

## Root display boundary

[display.rs](../service/parent/display.rs) now owns a shared service client for
root-to-display commands and immutable image grants. `output.rs` selects this
adapter at the root and the existing scoped interaction stream in nested matrices.
Presentation and owner scene updates submit/poll the same serialized shared client;
the service loop includes its notification and lifetime endpoint beside child and
storage interests. A pending request cannot be overwritten by another call.

The adapter's RefCell separates mutable transport bookkeeping from immutable
borrows of retained images in this single-threaded process. It does not permit
concurrent calls: Client rejects a second outstanding request. Synchronous cursor,
image replay and cleanup calls retain their existing scheduling behavior. There
is no copied-payload fallback after the display connection upgrades.

## Storage boundaries

[storage.rs](../service/parent/storage/storage.rs) owns scope/delegation decisions;
[provider.rs](../service/parent/storage/provider.rs) owns the two root/provider
shared clients and their replacement. [pending.rs](../service/parent/storage/pending.rs)
submits and polls forwarded or launch-read work with immutable attachments and
an absolute deadline. Setup and local Scope calls still wait synchronously.
The matrix waits on notification plus endpoint lifetime;
partial attachment expiry is included in that wait. Local storage operations borrow
the same client via Scope, so a temporary scope cannot reset transport IDs or close
an otherwise healthy provider connection. Late/ambiguous failures retire it.

Scope stamps, source selection and binding revocation are unchanged. Link slots
continue to define the provider's private roots.

[upstream.rs](../service/parent/storage/upstream.rs) owns the persistent shared
Client for a nested stock matrix's separate storage stream. Its local Scope,
forwarded requests and binding refreshes all use that connection. Parent servers
preselect the five child storage slots before admission; remove/reinstall resets
custody and correlation without requiring another static-slot borrow. Cleanup
tolerates endpoints already closed by shared transport. Late replies retain the
old incarnation and cannot reach the replacement child.

Each end reserves a writer page, reader page and notification allowance. The small
desktop fixtures explicitly fund both ends without taking the app's existing
allowance. Stored-program admission also counts the server-side transport cost
before requesting growth and lending resources to a child. Setup and local reads still wait synchronously. Omega-facing app
bindings and Split use shared storage too. Fixtures combining interaction and storage
on one pair retain copied packets; an upgraded pair never falls back to them.
