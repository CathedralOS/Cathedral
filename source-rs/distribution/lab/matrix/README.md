# Matrix-owned process tree

Run from the repository root:

```powershell
python tools/boot-harness-rs/run.py --matrix-test --timeout 120
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 120
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/matrix-desktop.json --release --window
```

The headless [profile](../../../../tools/boot-harness-rs/profiles/matrix.json)
reserves executables but has no boot-wired links. The distribution admits only the
root matrix, lending it the approved child launch grants. That matrix admits an
inner matrix and outside sibling; the inner admits its leaf. Only direct parents
hold child process handles and private ports. Ordinary parent connections are
created with IPC and installed in the child environment.

[main.rs](main.rs) selects distribution policy or a leaf. Follow
[scenario.rs](scenario.rs) through local layout, [confinement/input](scenario/checks.rs)
and [subtree replacement](scenario/recovery.rs). Owner commands can be routed down
local child slots for distribution policy/instrumentation. Providers forward
these asynchronously while continuing to service child requests: blocking on the
child's reply would deadlock its upward viewport query. Child connections cannot
invoke this owner route, spawn, stop, inspect siblings or read global pixels.

The checks preserve the original local-view, pointer capture, key attenuation,
revoke/resize and fresh-binding cases. Replacement now stops only the inner
matrix; the kernel reclaims its leaf. The parent reacquires the returned launch
budget and admits a fresh subtree while the outside sibling process survives.

## Visible integration

The [desktop policy](desktop.rs) supplies layout and initial appearance. The same
platform matrix implementation owns input, clipping and child admission at both
levels. The root environment additionally receives explicit compositor and
physical-source endpoints; these are the only device-facing adapters. Apps get
only their parent pair and a child-local wake receiver. The [app](desktop/app.rs) waits for local input/view invalidations and
submits an oversized rectangle that must be clipped at each ownership boundary.

The [surface validator](../../../platform/services/matrix/world/surface.rs)
translates one level, validates the binding epoch and prefixes the child-local
surface key. The root [render adapter](../../../platform/services/matrix/service/parent/rendering.rs)
sends flattened retained rectangles to the existing
[compositor](../../../platform/services/display/compositor.rs). No ancestor pixel
buffers are copied. Subtrees paint in the same order as hit testing. Placement,
revoke, stop and disconnect erase the affected retained branch. Apps periodically
repaint, so placement changes can briefly leave a cleared region; retained local
scene replay and event-driven delivery are still future work.

Click an app, then Enter to change color. The test injects real PS/2 mouse/keyboard
input, checks sibling focus isolation and clipped scanout, and verifies startup
subtree replacement. The physical source shares the existing controller/decoders;
its dedicated connection accepts only input operations, never owner commands.
The root owns the compositor connection; children have neither it nor a framebuffer.

This renderer is deliberately bounded: two local children, six surface keys for
two matrix levels, and one rectangle per path. It does not yet integrate the
older session's images, text, synthetic pointers or richer scene snapshots.
Device/root-provider recovery remains separate from the proved inner-subtree
recovery. [STATUS.md](../../../STATUS.md) tracks these boundaries and the kernel
metadata/resource audit. Protocol locality is not a complete timing-isolation claim.

The desktop retains app rectangles across owner placement changes. Its startup
probe checks that idle apps do not repaint or consume 20 ms of CPU per second.
Use `--matrix-desktop-test --pressure --release --timeout 120` to add a
non-yielding descendant inside a 20% nested CPU allowance. Physical-input tests
report scanout latency; current equal-turn scheduling can still take about a
second in that deliberately constrained subtree. The ordinary profile has no spinner.
