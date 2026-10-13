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
[scenario entrance](scenario/scenario.rs) through local layout, [confinement/input](scenario/checks.rs)
and [subtree replacement](scenario/recovery.rs). Owner commands can be routed down
local child slots for distribution policy/instrumentation. Providers forward
these asynchronously while continuing to service child requests: blocking on the
child's reply would deadlock its upward viewport query. Child connections cannot
invoke this owner route, spawn, stop, inspect siblings or read global pixels.

The checks preserve the original local-view, pointer capture, key attenuation,
revoke/resize and fresh-binding cases. Replacement now stops only the inner
matrix; the kernel reclaims its leaf. The parent reacquires the returned launch
budget and admits a fresh subtree while the outside sibling process survives.

## Storage integration

The separate `--matrix-storage-test --release` profile starts at
[storage/storage.rs](storage/storage.rs). It shares [fixture.rs](fixture.rs)'s
matrix-owned process tree and the same service binary, adding two private catalog
sources at the root. Tests cover recursive grants, sibling isolation, stale
binding revisions, revocation during transactions, child replacement and durable
data after restarting the tree/provider. No leaf has a disk or provider link.
See [storage scopes](../../../platform/services/matrix/world/storage/README.md)
for the bounded whole-root policy and protocol.

## Visible integration

The [desktop policy](desktop/desktop.rs) supplies layout and initial appearance. The same
platform matrix implementation owns input, clipping and child admission at both
levels. The root environment additionally receives explicit compositor and
physical-source endpoints; these are the only device-facing adapters. Apps get
only their parent pair and a child-local wake receiver. The [app](desktop/app/app.rs) waits for local input/view invalidations and
submits an oversized rectangle that must be clipped at each ownership boundary.

The desktop receives code-only catalogue grants and a funded process pool.
[Startup admission](desktop/admission.rs) stages these into the root; owner-only
RUN requests select and fund descendants using the same provider at each level.
The test checks rollback, nested pool exhaustion and repeated subtree replacement.
Device launches remain explicit static grants.

[Owned surfaces](../../../platform/services/matrix/world/surface/README.md) use
fresh child-local handles and conserving allowances. Each app owns a base rectangle
and an accent object. Every matrix clips/translates one level and preserves subtree
stacking; the root flattens into the existing compositor. Same-extent placement
replays retained content without app traffic. Epoch changes, release and teardown
retire objects; stale handles cannot paint their replacements. There are no
ancestor pixel-buffer copies.

Click an app, then Enter to change color. The test injects real PS/2 mouse/keyboard
input, checks sibling focus isolation and clipped scanout, and verifies startup
subtree replacement. The physical source shares the existing controller/decoders;
its dedicated connection accepts only input operations, never owner commands.
The root owns the compositor connection; children have neither it nor a framebuffer.

This renderer has two direct children per matrix and sixteen root surface objects.
It does not yet integrate the older session's images, text, synthetic pointers or
richer scene snapshots. Recursive stacking has no hard-coded two-level path keys.
Device/root-provider recovery remains separate from the proved inner-subtree
recovery. [STATUS.md](../../../STATUS.md) tracks these boundaries and the kernel
metadata/resource audit. Protocol locality is not a complete timing-isolation claim.

The desktop retains app rectangles across owner placement changes. Its startup
probe checks that idle apps do not repaint or consume 20 ms of CPU per second.
Use `--matrix-desktop-test --pressure --release --timeout 120` to add a
non-yielding descendant inside a 20% nested CPU allowance. Physical-input tests
report scanout latency; current equal-turn scheduling still takes roughly 0.5 to 0.9 seconds
in recent runs of that deliberately constrained subtree. The ordinary profile has no spinner.

Physical admission uses the same conserved frame funding as other processes.
Owner-only `RUN_MEMORY` stages the physical-frame allowance for the next `RUN`
attempt on an empty local child slot. An attempted admission consumes this setting
on failure as well as success; ordinary `RUN` defaults to 256 frames per requested
process account. This changes admission policy, never a running child's funding.
The [physical-memory guide](../../../kernel/core/user_tasks/resources/physical/README.md)
connects the kernel transaction, desktop failure/recovery probes and retained charges.
