# Recursive matrix boundary lab

Run from the repository root:

```powershell
python tools/boot-harness-rs/run.py --matrix-test --timeout 60
```

Add `--release` for optimized verification. This is a headless protocol fixture;
the existing session profile remains the visual desktop. Logs live in
`build/boot-harness-rs/matrix-test/serial.log`.

The [profile](../../../../tools/boot-harness-rs/profiles/matrix.json) boots an
outer and inner instance of the same
[matrix service](../../../platform/services/matrix/main.rs), plus one nested
leaf and one outside sibling. The inner matrix obtains its viewport and input
using exactly the child interface its leaf uses. Neither leaf has a launch grant
or a link to the other's world. The supervisor retains private test control
connections; they are instrumentation, not a child-facing environment API.

[main.rs](main.rs) selects the boot scenario or a leaf peer.
[scenario.rs](scenario.rs) admits tasks and supplies local layout, then descends
into [confinement/input checks](scenario/checks.rs) and
[revocation/replacement checks](scenario/recovery.rs). [leaf.rs](leaf.rs) sends
probes only to its own parent endpoint. Across IPC, follow the shared
[matrix protocol](../../../contracts/matrix.rs) to the service
[dispatch loop](../../../platform/services/matrix/service.rs).

The checks compare leaf replies while moving the parent and revoking/replacing
an outside sibling's grant. They reject owner/global queries, route pointer
capture through both worlds with local bounds, attenuate keys, discard queued
input on ancestor revoke, resize a viewport, and require fresh descendant
authorization after replacing the inner service. The original sibling process
stays alive. Host tests additionally cover rapid revoke/regrant, queue overflow,
held keys at focus changes and connection-incarnation replacement.

The root's grant and source events come from the fixture's root adapter. Physical
device attachment can supply that same boundary later; it does not require a
second input manager. Layout choices belong to this distribution fixture, while
the platform enforces the local grant and routes within it.

The next integration steps are local surface submission into the shared
compositor, distribution matrix instances owning their child layout and
lifecycle, and kernel support for confined child admission. Rendering need not
copy pixels through every ancestor. See the provider
[charter](../../../platform/services/matrix/CHARTER.md) for the precise current
limits, including the remaining kernel metadata and timing audit.
