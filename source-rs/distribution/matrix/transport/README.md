# Concurrent input and presentation

The stock matrix keeps its existing scoped interaction connection. It permits
one tagged input poll beside one asynchronous presentation exchange. No global
locator, new child capability, kernel queue size or public packet shape changes.

Follow [the service loop](../service/service.rs) into
[interaction.rs](../service/parent/interaction.rs), the sole upstream reply reader
while these exchanges are active. [transport.rs](transport.rs) distinguishes
tagged snapshots from canonical untagged presentation status. This concurrent
client uses poll IDs 1 through `u32::MAX - 1`, never reuses them and reserves the
high words used by signed status. The public serial poll protocol is unchanged.
Unmatched poll tags cannot satisfy either pending exchange; malformed status is
fatal rather than an uncertain success.

[polling.rs](../service/parent/polling.rs) applies input snapshots;
[presentation.rs](../service/parent/presentation.rs) retains the scene and advances
begin/update/commit/abort. Local input queries remain serviceable even from the
publishing child. Startup holds and authority-changing requests still wait.
A snapshot replacing the view or surface capacity is retained until publication
ends, then consumed before parking. Malformed input that invalidates a publishing
binding fails closed. A slow reply alone neither revokes input nor replays work.

Kernel IPC channels hold one message. The matrix therefore opts into the runtime
server's [bounded copied-reply retention](../../../platform/libraries/user-runtime/server/replies.rs):
one kernel reply plus one software reply per client. A full software slot pauses
request intake for that client. Connection stamps discard stale replies; excessive
pipelining disconnects that client. The arena belongs to this process and has a
fixed size. Other servers retain their existing behavior unless they opt in.
Endpoint/memory transfers still require their caller's explicit custody handling.
Request sends also tolerate full channels. Idle retry uses a one-tick deadline;
writable-channel wakeups are not implemented. There is no extra acknowledgement
round trip in the ordinary path.

`desktop_test.py --nested-desktop --interaction-delay --timeout 240` holds a real
presentation for five seconds and requires fresh keyboard input in every matrix
realm on the three-level focused chain before release, on fresh and cold boots.
It also exercises delayed polls/images, stale tags, protected close, saved Notes
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

## Root storage boundary

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
continue to define the provider's private roots. Nested storage hops and Omega
application bindings have not migrated; copied payloads there are remaining work.
