# Independent Omega matrix

Start at [matrix/main.omg](matrix/main.omg). [session.omg](matrix/session.omg)
divides its granted area into three regions, admits children with different supplied
services and places them. [requests.omg](matrix/requests.omg) chooses owner,
child, service-acquisition and disconnection handling. No stock desktop,
launcher, configuration object, taskbar or filesystem is required.

The [native consumer](native/main.omg) packages the same ABI-1 state/step layout
used by other Omega components. Its **operation vocabulary is private to this
proof**, in [bridge.omg](matrix/bridge.omg). It is not the common application host
protocol and has not been blessed as a general matrix-server API.

The [Rust adapter](../../../../source-rs/distribution/lab/matrix-composition/matrix/host/host.rs)
executes requested mechanisms: the platform World for scoped geometry/input,
runtime admission for endpoints/tasks, and the server for authenticated requests.
It never recovers a child identity from message words. The retained Request owns
reply custody; words 10-13 report local caller slot/incarnation, event kind and
length separately from the eight untrusted message words. Slot 3 means the
supplied owner endpoint. Omega chooses the grant bits before admission; service
offers transfer only those preallocated rights. The proof-only relay and rollback
operations reuse the existing test harness and are explicitly not runtime APIs.

`NEXT` cannot discard an unanswered request. Child dispatch uses the retained
endpoint identity even if an app changes packet metadata. Requests carrying a
memory object are reported separately after releasing custody; this program
rejects them. Parent death still invokes the kernel's existing subtree teardown.

Run the same live IPC proof against both implementations:

```powershell
python tools/boot-harness-rs/matrix_composition_test.py
python tools/boot-harness-rs/matrix_composition_test.py --omega
python tools/boot-harness-rs/matrix_composition_test.py --upstream
cd source-rs
cargo test --locked -p cathedral-three-peer-matrix --lib --features omega
```

The Omega profile grants nine private pages for the component's state/packet.
The original Rust proof remains the comparison path. Six compiled policy tests
cover startup refusal, caller separation, service-grant choice on replacement,
malformed requests, disconnection, admission errors and parent refresh without
losing authenticated request custody. The guest proof checks
actual service transfers, forged/forbidden requests, sibling isolation, stale
views, repeated replacement, interrupted transfers, failed-admission rollback
and whole-subtree reclamation.

This proves a matrix program in Omega over existing Rust platform mechanisms.
Its storage grant exercises endpoint custody only; it does not serve a filesystem
or distinguish persistent realms. It has a virtual input space, no framebuffer
and no rendering. The stock F5 matrix still owns its
Rust supervision and forwarding loop. Its configuration, close/loss, startup and
[launcher recovery workflow](../../matrix/README.md) execute in Omega. The bridge stays private after this second composition;
its synchronous upstream exchange is not a reusable desktop service loop.

## Supplied upstream connection

The same program runs under a real Rust fixture parent with `--upstream`.
[The nested proof](../../../../source-rs/distribution/lab/matrix-composition/nested/nested.rs)
admits that parent, which lends the Omega matrix its three leaf launch grants.
The matrix receives only a scoped broker pair in its environment. Startup word
14 reports whether that binding exists; Omega saves that fact before processing
untrusted requests. An empty environment selects the original top-level proof.
An invalid or refused supplied parent never falls back to a synthetic grant.

[host/parent.rs](../../../../source-rs/distribution/lab/matrix-composition/matrix/host/parent.rs)
acquires the supplied interaction service once and applies atomic POLL snapshots
through the shared World. INITIALIZE returns the actual epoch, width, height and
key grant; Omega chooses three equal placements inside it. REFRESH preserves the
local request and caller metadata. A changed parent epoch invalidates descendant
placements, including on resize. Explicit placement must establish fresh grants;
restoring a revoked parent does not resurrect old placements or queued input.

The live fixture checks two coordinate translations, sibling isolation, queued
press/release ordering, key attenuation, forged owner operations, resize clipping
after explicit placement, stale epochs and revoke/regrant. It cancels and restarts
the matrix three times per parent while checking resource counters. The parent
then returns with the matrix and peers alive; the supervisor starts the complete
tree again with the returned loans and unchanged charges.

Limits: snapshots are requested before local dispatch, with the existing
retirement backpressure. This proof does not consume notification wakes or run a
frame loop, and a slow parent can delay local dispatch. The stock desktop's
asynchronous parent handling must be preserved when moving a lifecycle branch
to Omega. Neither rendering latency nor wake-driven input delivery is measured.
