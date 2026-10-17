# Cathedral desktop

The default boot enters [main.rs](main.rs), [session](session/session.rs), then
[shell policy](session/shell/shell.rs). The [launcher](session/shell/launcher.rs)
reports local input choices; the owner admits Notes or Colors in a nested matrix,
protects unsaved notes and supervises storage. See the [distribution map](../README.md)
for F5, controls, composition and the same-image interactive test.

The older proofs below remain explicitly selected features/profiles. This package
was promoted from `distribution/lab/matrix`; its default composition now uses the
`shell` feature rather than booting a fixed pair of test apps.


Run from the repository root:

```powershell
python tools/boot-harness-rs/run.py --matrix-test --timeout 120
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 120
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/matrix-desktop.json --release --window
```

The headless [profile](../../../tools/boot-harness-rs/profiles/matrix.json)
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
matrix-owned process tree and the same service binary, adding two private object
roots at the root. Tests cover recursive grants, sibling isolation, stale
binding revisions, revocation during transactions, child replacement and durable
data after restarting the tree/provider. No leaf has a disk or provider link.
See [storage scopes](../../platform/services/matrix/world/storage/README.md)
for the bounded whole-root policy and protocol.

## Visible integration

The [desktop policy](session/session.rs) supplies layout and initial appearance. The same
platform matrix implementation owns input, clipping and child admission at both
levels. The root environment additionally receives explicit compositor and
physical-source endpoints; these are the only device-facing adapters. Apps get
only their interaction pair, catalog pair and a child-local wake receiver. The [app](session/app/app.rs) waits for local input/view invalidations and
submits an oversized rectangle that must be clipped at each ownership boundary.

The desktop receives code-only catalogue grants and a funded process pool.
[Startup admission](session/admission.rs) stages these into the root; owner-only
RUN requests select and fund descendants using the same provider at each level.
The test checks rollback, exhausted frame funding and repeated subtree replacement.
Device launches remain explicit static grants.

[Owned surfaces](../../platform/services/matrix/world/surface/README.md) use
fresh child-local handles and conserving allowances. Each app owns a base rectangle
and an accent object. Every matrix clips/translates one level and preserves subtree
stacking; the root flattens into the existing compositor. Same-extent placement
replays retained content without app traffic. Epoch changes, release and teardown
retire objects; stale handles cannot paint their replacements. There are no
ancestor pixel-buffer copies.

Click an app, then Enter to save and change color. The ordinary profile keeps a
private disk at `build/boot-harness-rs/matrix-desktop-storage.raw`; both apps restore
committed colors on the next boot. Each app uses object 2 in its own root (object 1
in the first root is reserved by the store's legacy empty-record initialization).
No app receives a provider link or disk grant.

`--matrix-desktop-test` enables `desktop-probes`, using a disposable disk and
[recovery checks](session/recovery.rs) for revoked writes, stale bindings, termination
with staged pixels/data, focus loss and sibling survival. It then starts a fresh
QEMU process over the same disk and checks both saved colors. Ordinary window boots
skip these destructive probes. The first test boot log is `first-boot-serial.log`;
`serial.log` contains the reboot.

The test injects real PS/2 mouse/keyboard
input, checks sibling focus isolation and clipped scanout, and verifies startup
subtree replacement. The physical source shares the existing controller/decoders;
its dedicated connection accepts only input operations, never owner commands.
The root owns the compositor connection; children have neither it nor a framebuffer.

This renderer has two direct children per matrix and sixteen root surface objects.
It supports sealed RGBX images, but does not yet integrate the older session's text, synthetic pointers or
richer scene snapshots. Recursive stacking has no hard-coded two-level path keys.
[Provider recovery](session/providers.rs) explicitly rebuilds the root tree after
input/display loss. [Storage stalls](session/stalls.rs) prove sibling rendering
through blocked saves and reconcile unacknowledged commits after provider replacement.
The [nested peer](session/stalls_peer.rs) shares the saving app's immediate matrix;
its pointer/key input and repaint complete before the storage deadline. It also
survives catalog revocation, provider replacement and its sibling's death. Its
state changes are deliberately volatile because only the saver holds the inherited
catalog grant. [STATUS.md](../../STATUS.md) tracks these boundaries and the kernel
metadata/resource audit. Protocol locality is not a complete timing-isolation claim.

The desktop retains app rectangles across owner placement changes. Its startup
probe checks that idle apps do not repaint or consume 20 ms of CPU per second.
Use `--matrix-desktop-test --pressure --release --timeout 180` to add a
non-yielding descendant inside a 20% nested CPU allowance. Physical-input tests
report scanout latency. This nine-process profile now includes storage, provider
recovery and cold reboot alongside the spinner. During the nested interaction
proof the second nested slot temporarily hosts the interactive peer in place of
the spinner, retaining the matrix's 20% cap. The spinner is restored before the
host physical-input check. The ordinary profile reserves the same four descendant
slots, uses three outside probes, and has no spinner.

Channel funding remains within the fixed 32-channel arena: four for distribution
adapter connections, four reserved for display, four for input, zero for storage,
and twenty delegated to the root. The root retains ten and lends ten to its inner
matrix; each child bundle costs five channels. The largest construction needs
20 local dynamic handles (five incoming, five retained for an existing child,
ten for the new child before transfer). Boot control/provider links use their
separate static backing.
Add `--trace-input` for elapsed/CPU diagnostics; these are not latency guarantees.
Compact rectangle packets avoid BEGIN/stage/COMMIT round trips at every level;
publication waits now return to the service loop. Neither change introduces CPU
priority, budget donation or a hard latency guarantee.

Physical admission uses the same conserved frame funding as other processes.
Owner-only `RUN_MEMORY` stages the physical-frame allowance for the next `RUN`
attempt on an empty local child slot. An attempted admission consumes this setting
on failure as well as success; ordinary `RUN` defaults to 256 frames per requested
process account. This changes admission policy, never a running child's funding.
The [physical-memory guide](../../kernel/core/user_tasks/resources/physical/README.md)
connects the kernel transaction, desktop failure/recovery probes and retained charges.

## Database desktop and save recovery

The [object store guide](../../platform/libraries/database-objects/README.md)
owns the store; the provider's recovery and timing proofs are driven by
`matrix_database_test.py`. The ordinary desktop
uses [persistence entrance](session/persistence/persistence.rs) to observe provider death,
retired root streams and bounded health probes, reopen it,
and explicitly restore root and nested grants. Apps remain alive; their
[state entry](session/app/state.rs) preserves confirmed values and displays a red
mark on failed saves. Enter reloads an unresolved save rather than toggling again.
The test-only [database recovery entry](session/database_recovery.rs) sequences
startup timeout, before/after-commit cuts, sibling interaction and exact readback.
Its temporary peer is admitted before the startup save is gated, so child admission
does not itself wait on that storage request.

`trace-storage` marks app send/reply, matrix receipt/admission/forward/reply and
provider phases. Calls are omitted in ordinary builds. This adds neither kernel
storage policy nor global names to child views. A timeout alone does not authorize
an app to revive a retired binding. The owner has three replacement attempts per
session, then leaves storage offline; a failed reopen also stops recovery. A lost
app transport requires replacement. The standalone restart-budget policy test is:

```powershell
rustc --edition 2024 --test source-rs/distribution/session/session/persistence/policy.rs -o build/recovery-policy.exe
./build/recovery-policy.exe
```

The `notes` composition substitutes the separate [notes application](../applications/notes/README.md)
for the color probe and explicitly funds its retained/replacement image pages.
Its app protocol and draft state stay in that application, not in the matrix service.
