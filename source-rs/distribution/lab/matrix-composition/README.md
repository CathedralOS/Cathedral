# Independent matrix composition

This proof boots a different matrix executable with three equal virtual input
regions. It has no launcher/app roles, saved desktop configuration, framebuffer,
storage provider or dependency on the stock matrix contract. The layout checker
rejects stock distribution dependencies.

Two matrix programs run the same proof. The standalone
[Omega matrix](../../../../source/distribution/lab/matrix-composition/README.md)
is the migration target; the Rust program here is the comparison path.
Start at [matrix/main.rs](matrix/main.rs), then [matrix/session.rs](matrix/session.rs).
That program chooses its arrangement and serves three peer connections using
the existing [runtime server](../../../platform/libraries/user-runtime/server/server.rs).
The [shared local commands](../../../platform/libraries/matrix/world/commands.rs)
apply owner placement/revocation and trusted-source input. Child packets take a
separate `World::request` path, with identity supplied by the accepted endpoint.
The stock F5 matrix uses these same commands behind its own authentication and
policy dispatch.

The [proof entrance](main.rs) selects the [supervisor](proof.rs) or a
[child relay](peer.rs). The supervisor lends three launch grants to the matrix;
only the matrix admits and controls those children. The Rust matrix's
[children.rs](matrix/children.rs) uses
[shared runtime admission](../../../platform/libraries/user-runtime/matrix/admission/README.md)
to install one scoped broker connection. No static peer links remain. The proof
tests local views, malformed/forbidden requests, input confined to the third
region, revocation and connection replacement. Replacement cannot inherit the old
grant, and unrelated peers keep their views. The relays have no launch or device
authority and no peer links. Sixteen busy spawn failures and sixteen partial
allocation failures preserve object charges and notification handle counts.
Sixteen replacements reuse the middle slot without changing siblings. Killing
the matrix collects its whole subtree and returns the loans for a fresh matrix.
The Rust matrix additionally proves that a full notification handle table forces
wake creation to fail after channel allocation and that sixteen such attempts
unwind exactly.

The [service acquisition contract](../../../platform/contracts/services/README.md)
is exercised by every peer. The third peer can acquire a storage pair; the other
two cannot. These storage endpoints exercise custody only, not a filesystem or
storage provider. Unknown/ungranted services, incompatible versions and forged
identity fields refuse. Repeat wire acquisition cannot duplicate a transferred
right; repeat runtime calls return the cached binding. Sixteen cycles cancel a
child before any acquisition or halfway through storage transfer, then replace
it without consuming its siblings' resources. Peers assert that the supplied wake
is a notification, cannot signal or close it, and gain exactly one local wait right
despite repeated cached acquisition. The Rust matrix has thirteen transport
allowances: eleven for the three children's channels/notifications and two
spare for partial allocation rollback.

The `omega-matrix` fixture feature switches the owner in [main.rs](main.rs) and
the peers to the shared v4 protocol, which is the only one the Omega matrix
speaks; without it they keep copied broker and control payloads for the Rust
matrix. The two Omega profiles build every fixture entry with that feature.

The lab's owner probe envelope routes to a child with the high 32-bit word of
the opcode (slot + 1); the low word remains its requested operation. This is
local test orchestration, not a new platform protocol. [probes.rs](matrix/probes.rs)
waits asynchronously so a probed child can call its matrix. An expired or failed
probe ends this proof program; it is not the stock application's recovery policy.

From the repository root:

```powershell
python tools/boot-harness-rs/matrix_composition_test.py
python tools/boot-harness-rs/matrix_composition_test.py --omega
python tools/boot-harness-rs/matrix_composition_test.py --upstream
```

The harness uses separate firmware/ESP files and no storage disk. This is a
headless architecture check, not another user-facing desktop. F5 still runs the
normal desktop. Drawing, storage forwarding and alternative dynamic-reservation
policy remain outside this proof. Those paths remain in the stock program and
are candidates for extraction when another consumer needs them.

## Parent-backed proof

`matrix_composition_test.py --upstream` selects a separate profile and the
`upstream-proof` fixture feature. Start at [nested/nested.rs](nested/nested.rs):
it runs a real [parent provider](nested/parent.rs), which admits the Omega matrix
and lends three leaf launches. [behavior.rs](nested/behavior.rs) exercises both
IPC edges. The fixture parent uses the platform World, but never directly changes
the inner World or simulates a leaf reply. It connects owner control right after
admission and serves the matrix's broker only while an owner call is outstanding,
which is why the Omega matrix accepts owner control before it opens that broker.

The supplied region is offset (100,80), size 240x450 and Enter-only. The matrix
derives three 240x150 regions. The proof checks translation, queued key edges,
withheld keys, forbidden child commands, shrink/revoke/regrant and old-epoch
rejection. It repeatedly kills the matrix with its leaves alive and checks
resource recovery. The parent finally exits with a live subtree; the root
supervisor restarts that entire composition using the returned launch loans.
