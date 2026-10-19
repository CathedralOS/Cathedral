# Bootstrap admission

[admission.rs](admission.rs) shows the F5 retry boundary, then display admission,
root admission/policy and physical input. The catalogue lab calls
the same admission and input mechanisms, with its own storage-first proof setup.

Follow [display.rs](display.rs) for the initial INFO exchange, dimensions and
background, then [root.rs](root.rs) for funding and the endpoint handoff. Root
budgets and code authority are explicit in [funding.rs](funding.rs).
[input.rs](input.rs) checks source attenuation, transfers the source endpoints,
and waits for exactly `ready`. All readiness/request waits here are bounded by
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
page/channel limits, page/reader charges and dynamic handles against its baseline.
Failed cleanup is an invariant failure, not permission to start another attempt.

On exhaustion, input/root failures can still use the original display launch
grant to show a static restart message, without a matrix, input or storage. That
owner blocks while the display serves. If display cannot start, the diagnostic is
serial-only and the initial program exits. A driver replacement for a live user
is a different recovery operation; this path never destroys a live session.

The `bootstrap-*` features and service `startup_probe.rs` files are fault fixtures
only. `bootstrap_test.py` drives pre-initialization exits, stalls, malformed
replies and a refusal after staging but before a successful endpoint move. None
is part of the ordinary distribution profile. Hardware initialization itself is
not exhaustively fault-injected by these tests.
