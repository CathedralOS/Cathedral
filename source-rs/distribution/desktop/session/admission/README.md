# Bootstrap admission

[admission.rs](admission.rs) shows the F5 retry boundary, then display admission,
root admission/policy and physical input. The catalogue lab calls
the same admission and input mechanisms, with its own storage-first proof setup.

Follow [display.rs](display.rs) for the initial INFO exchange, dimensions and
background, then [root.rs](root.rs) for funding and the endpoint handoff.
F5 uses the display's bootstrap port for a short-lived shared configuration
connection. The presentation pair passed to the root is separate and untouched;
the root opens its own shared stream. [Attempt](attempt.rs) retains configuration
through root readiness and drops it on commit, or after collecting failed tasks.
The [display configuration dispatcher](../../../../platform/services/display/compositor/configuration.rs)
permits INFO, background, cursor enable and text on that boot port, not scene
publication, input injection or client grants. No request identity is rewritten
for reply dispatch. Copied positional labs keep their staged presentation pair.
Root
budgets and code authority are explicit in [funding.rs](funding.rs).
F5 opens one [shared root control](../control.rs), validates HEALTH and moves that
owner into the admitted Session. The connector owns its raw endpoints on success
and failure; the partial-attempt raw inventory never also owns them. Setup can
park here because the boot owner serves nothing the root needs; the root's display
peer runs independently. Setup and first HEALTH share the original 300-tick bound.
Later shell/persistence requests borrow that retained client. Its failures retire
the stream without replaying mutations; positional labs keep a copied adapter.
[input.rs](input.rs) transfers untouched source endpoints, opens an independent
shared readiness connection and checks an exact input HEALTH reply. The
[driver](../../../../platform/services/input/matrix.rs) checks denied SPAWN,
READ_PIXEL and BIND on its own shared source connection, initializes hardware and
publishes the initial pointer before serving HEALTH. Its
[startup control](../../../../platform/services/input/readiness.rs) flushes replies
until owner closure (bounded to 300 ticks), releases custody, then leaves physical
delivery on the independent source stream. No startup polling enters the input loop.
Both F5 and positional matrix labs use this readiness protocol.
All readiness/request waits here are bounded by
300 guest ticks. Stored program lookup and volume recovery have their own longer
deadlines in the shell and persistence subsystems.

[attempt.rs](attempt.rs) holds partial admission. Successful spawn removes moved
endpoints from that inventory; failed spawn retains them. Cleanup collects input,
root and display in that order, then closes only the endpoints still held by the
owner. [state.rs](state.rs) owns the admitted session and its later teardown.

F5 gets two attempts before storage or users start. Rollback restores root process
and physical reservations and its fixed page/channel allowance. The page/channel
allowance cannot all be merged into the owner: the combined total exceeds its
per-account ceiling. The owner compares process slots, physical funding/charges,
page/channel limits, page/reader charges, dynamic handles and notification
objects/rights against its baseline.
Failed cleanup is an invariant failure, not permission to start another attempt.

On exhaustion, input/root failures can still use the original display launch
grant and shared bootstrap configuration to show a static restart message,
without a matrix, input or storage. That
owner blocks while the display serves. If display cannot start, the diagnostic is
serial-only and the initial program exits. A driver replacement for a live user
is a different recovery operation; this path never destroys a live session.

The `bootstrap-*` features and service `startup_probe.rs` files are fault fixtures
only. `bootstrap_test.py` drives pre-initialization exits, stalls, malformed
replies and a refusal after staging but before a successful endpoint move. Display/root/input fixtures distinguish malformed setup from bad INFO/HEALTH after
valid shared setup; successful probe boots also check refused scene publication
and input injection on bootstrap display control. None
is part of the ordinary distribution profile. Hardware initialization itself is
not exhaustively fault-injected by these tests.
