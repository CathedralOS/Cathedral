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
[peer relay](peer.rs). The supervisor admits the separate matrix and three peers
using explicit boot-approved links. It tests local views, malformed/forbidden
requests, input confined to the third region, revocation and connection replacement.
Replacement cannot inherit the old grant, and unrelated peers keep their views.
The relays have no launch or device authority and no other peer link.

From the repository root:

```powershell
python tools/boot-harness-rs/matrix_composition_test.py
```

The harness uses separate firmware/ESP files and no storage disk. This is a
headless architecture check, not another user-facing desktop. F5 still runs the
normal desktop. These peers are siblings admitted by the test supervisor; this
does not yet prove independent matrix-owned child admission, nested upstream
adaptation, drawing or storage forwarding. Those orchestration paths remain in
the stock program and are candidates for extraction when another consumer needs
them. Shared mechanisms should grow from those actual requirements.
