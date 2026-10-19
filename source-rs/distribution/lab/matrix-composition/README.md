# Independent matrix composition

This proof boots a different matrix executable with three equal virtual input
regions. It has no launcher/app roles, saved desktop configuration, framebuffer,
storage provider or dependency on the stock matrix contract. The layout checker
rejects distribution dependencies in both proof packages.

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
to install explicit startup connections. No static peer links remain. It tests local views, malformed/forbidden
requests, input confined to the third region, revocation and connection replacement.
Replacement cannot inherit the old grant, and unrelated peers keep their views.
The relays have no launch or device authority and no peer links. Sixteen busy
spawn failures and sixteen partial allocation failures preserve channel charges.
Sixteen replacements reuse the middle slot without changing siblings. Killing
the matrix collects its whole subtree and returns the loans for a fresh matrix.

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
connection rollback. It does not prove nested upstream adaptation, drawing,
storage forwarding or alternative dynamic-reservation policy. Those paths remain in
the stock program and are candidates for extraction when another consumer needs
them. Shared mechanisms should grow from those actual requirements.
