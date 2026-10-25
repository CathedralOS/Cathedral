# Independent Omega matrix

Start at [matrix/main.omg](matrix/main.omg). [session.omg](matrix/session.omg)
chooses the virtual arrangement, admits three children with different supplied
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
cd source-rs
cargo test --locked -p cathedral-three-peer-matrix --lib --features omega
```

The Omega profile grants nine private pages for the component's state/packet.
The original Rust proof remains the comparison path. Four compiled policy tests
cover startup refusal, caller separation, service-grant choice on replacement,
malformed requests, disconnection and admission errors. The guest proof checks
actual service transfers, forged/forbidden requests, sibling isolation, stale
views, repeated replacement, interrupted transfers, failed-admission rollback
and whole-subtree reclamation.

This proves a matrix program in Omega over existing Rust platform mechanisms.
Its storage grant exercises endpoint custody only; it does not serve a filesystem
or distinguish persistent realms. It has a virtual input space, no framebuffer,
no upstream matrix adapter and no rendering. The stock F5 matrix still owns its
Rust supervision and forwarding loop; only its configuration/placement policy
has moved to Omega. Next growth should prove an actual upstream/child path
before promoting this proof's bridge into platform infrastructure.
