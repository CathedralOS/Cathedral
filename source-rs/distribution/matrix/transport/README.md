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
startup handoff. Root physical-source dispatch, image calls, switching existing
apps and teardown/compensation retain synchronous work. An atomic scene
still serializes visual commits. This is not a claim of end-to-end latency bounds
or that sibling pixels can commit into another app's open scene transaction.
