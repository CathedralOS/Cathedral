# Independent matrix composition

This proof boots a different matrix executable with three equal virtual input
regions. It has no launcher/app roles, saved desktop configuration, framebuffer,
storage provider or dependency on the stock matrix contract. The layout checker
rejects stock distribution dependencies; the Omega native consumer may depend
on its own proof package.

Start at [matrix/main.rs](matrix/main.rs), then [matrix/session.rs](matrix/session.rs).
That program chooses its arrangement and serves three peer connections using
the existing [runtime server](../../../platform/libraries/user-runtime/server/server.rs).
The [shared local commands](../../../platform/libraries/matrix/world/commands.rs)
apply owner placement/revocation and trusted-source input. Child packets take a
separate `World::request` path, with identity supplied by the accepted endpoint.
The stock F5 matrix uses these same commands behind its own authentication and
policy dispatch. No new generic server framework was needed for this subset.

The [proof entrance](main.rs) selects the [supervisor](proof.rs) or a
[child relay](peer.rs). The supervisor lends three launch grants to the matrix;
only the matrix admits and controls those children. Its [children.rs](matrix/children.rs)
uses [shared runtime admission](../../../platform/libraries/user-runtime/matrix/admission/README.md)
to install one scoped broker connection. No static peer links remain. It tests local views, malformed/forbidden
requests, input confined to the third region, revocation and connection replacement.
Replacement cannot inherit the old grant, and unrelated peers keep their views.
The relays have no launch or device authority and no peer links. Sixteen busy
spawn failures and sixteen partial allocation failures preserve object charges
and notification handle counts. With object funding spare, a full notification
handle table separately forces wake creation to fail after channel allocation;
sixteen attempts unwind exactly, and subsequent admissions still work.
Sixteen replacements reuse the middle slot without changing siblings. Killing
the matrix collects its whole subtree and returns the loans for a fresh matrix.

The [service acquisition contract](../../../platform/contracts/services/README.md)
is exercised by every peer. The third peer can acquire a storage pair; the other
two cannot. These storage endpoints exercise custody only, not a filesystem or
storage provider. Unknown/ungranted services, incompatible versions and forged
identity fields refuse. Repeat wire acquisition cannot duplicate a transferred
right; repeat runtime calls return the cached binding. Sixteen cycles cancel a
child before any acquisition or halfway through storage transfer, then replace
it without consuming its siblings' resources. Peers assert that the supplied wake
is a notification, cannot signal or close it, and gain exactly one local wait right
despite repeated cached acquisition. The matrix has thirteen transport allowances:
eleven for the three children's channels/notifications and two spare for partial
allocation rollback. These peers keep copied broker payloads as an explicit
fixture; ordinary F5 uses shared broker payloads with the same wait-right handoff.

The lab's owner probe envelope routes to a child with the high 32-bit word of
the opcode (slot + 1); the low word remains its requested operation. This is
local test orchestration, not a new platform protocol. [probes.rs](matrix/probes.rs)
waits asynchronously so a probed child can call its matrix. An expired or failed
probe ends this proof program; it is not the stock application's recovery policy.

From the repository root:

```powershell
python tools/boot-harness-rs/matrix_composition_test.py
```

The harness uses separate firmware/ESP files and no storage disk. This is a
headless architecture check, not another user-facing desktop. F5 still runs the
normal desktop. This now proves matrix-owned template admission with runtime
connection rollback. The separate Omega upstream profile below adds nested
input/view adaptation. Drawing, storage forwarding and alternative
dynamic-reservation policy remain outside this proof. Those paths remain in
the stock program and are candidates for extraction when another consumer needs
them. Shared mechanisms should grow from those actual requirements.

## Omega implementation of the same matrix

`matrix_composition_test.py --omega` runs this unchanged supervisor/peer proof
against [the Omega matrix](../../../../source/distribution/lab/matrix-composition/README.md).
The `omega` feature selects [host/host.rs](matrix/host/host.rs) instead of the Rust
session. Omega owns admission order, service-grant choices, layout, dispatch and
lifecycle requests. The adapter executes them through the same platform World,
runtime server and admission library used above. It retains authenticated Request
custody across the native call; message contents cannot supply caller identity.
Nine private pages are explicitly funded in the separate profile.

The original Rust session remains the default comparison proof. The Omega bridge
is deliberately local to this lab, including its probe relay and rollback check;
those operations do not enter the common app host or claim to be a final runtime.
Storage is still only a custody proof, not persistent realm forwarding. F5's
stock matrix is not yet running this event loop.

## Parent-backed Omega proof

`matrix_composition_test.py --upstream` selects a separate profile and the
`upstream-proof` fixture feature. Start at [nested/nested.rs](nested/nested.rs):
it runs a real [parent provider](nested/parent.rs), which admits the Omega matrix
and lends three leaf launches. [behavior.rs](nested/behavior.rs) exercises both
IPC edges. The fixture parent uses the platform World, but never directly changes
the inner World or simulates a leaf reply.

The supplied region is offset (100,80), size 240x450 and Enter-only. Omega derives
three 240x150 regions. The proof checks translation, queued key edges, withheld
keys, forbidden child commands, shrink/revoke/regrant and old-epoch rejection.
It repeatedly kills the matrix with its leaves alive and checks resource recovery.
The parent finally exits with a live subtree; the root supervisor restarts that
entire composition using the returned launch loans.

The [parent adapter](matrix/host/parent.rs) is deliberately synchronous and
request-driven. It preserves World retirement backpressure and serialized reply
custody; it does not demonstrate desktop latency or asynchronous notification
handling. See the [Omega guide](../../../../source/distribution/lab/matrix-composition/README.md)
for the private bridge and current limits.
