# Cathedral desktop

The default boot enters [main.rs](main.rs), then [session/session.rs](session/session.rs):
admit display and the root matrix, configure workspace policy, start input, open
storage, run the chooser over the folder of users, request each user
matrix's own startup and supervise storage. [shell/](session/shell/shell.rs)
resolves `Login` and `Users` by path in the supplied root.
[Root control](session/control.rs) retains one shared connection for admission,
shell commands and storage regrant. Session helpers borrow it; they never rebuild
a client over copied raw handles. [Display admission](session/admission/display.rs)
uses a separate, temporary shared configuration port, leaving presentation endpoints
untouched for the root's own stream. It drops configuration after root readiness.
Input startup uses its own shared
HEALTH connection, dropped after acceptance; the driver retains only its source
connection for ordinary delivery.
The owner funds four writer pages, two reader pages and eight channel/notification
allowances: four transferred display/source channels, root/storage control and
bounded overlap with retiring display and input-startup connections. Remote
readers and signal rights can keep their backing charged after local drop. Kernel ceilings are unchanged.
[Colors](../applications/colors/main.rs) is a separate installed program. See the
[distribution map](../README.md) for F5, controls, composition and the
same-image interactive test.

A failed user `CONFIGURE` returns to a fresh picker with CANNOT OPEN. Cleanup
accepts an already-collected child, so the root survives a race between startup
failure and normal task collection. App admission refusals and crash observations
stay in the launcher process; the desktop does not inspect user grandchildren.
Missing root Login/Users, rejected picker code and picker death enter the
[login recovery controller](session/shell/login.rs). It runs the boot-bundled
[recovery program](recovery/main.rs), which asks for an explicit retry without
holding a storage realm. [Storage startup](session/persistence/startup.rs) runs
after display/input and polls readiness without blocking their event loops. A
3,000-tick timeout, process death or non-`ready` response retires the task and its
control stream and enables a single explicit retry.
[Bootstrap admission](session/admission/admission.rs) gives initial display, root
and input two attempts before storage or users start. Each failed attempt collects
its tasks and closes its retained endpoints before restoring root funding and
checking the owner's resource snapshot. Display/root requests and input readiness
have a 300-tick deadline and validate reply contents. Persistent root/input failure
starts display alone with **COULD NOT START / PLEASE RESTART**; persistent
display failure is reported to the launch terminal. There is no automatic loop
and no attempt to reconstruct a running user through this path.
The matrix now retries a failed configured launcher once without restarting its app.

Colors' [close control](../applications/colors/session/closing.rs) renders an X and handles local
pointer clicks. A clean close returns from the app; the matrix reclaims the
child and its launch slot through its existing exit path. An uncertain storage
result requires a second, explicit CLOSE ANYWAY click. Its LOSS reply also
reports that uncertainty, so the chooser cannot silently close it.

[Shell handoff](session/shell/shell.rs) stops and collects the picker as soon as
its sibling user matrix is admitted, before loading the user launcher from
storage. This retires the picker's display, input and storage binding; it does
not leave an invisible login process behind. The
[user matrix's configuration loader](../../distribution/matrix/service/children/configuration.rs)
places the separate launcher and app using its own saved arrangement. The user matrix owns
the full display, with its launcher at the left and its app in the remaining
space. The root keeps supervising and forwarding without visible boot chrome.
If the user matrix is lost, the owner starts a fresh picker; storage recovery
does not regrant the retired picker. A missing storage binding alone is not
process death: the owner retains the remembered realm unless the child's
control route confirms PEER_CLOSED. This keeps a storage failure from
restarting login over a live user and preserves the realms needed for recovery.
The [replacement controller](session/persistence/replacement.rs) polls fresh
provider readiness without blocking the F5 owner. Its separate 30-second bound
allows unclean-volume recovery; the per-operation deadline remains six seconds.
Only accepted readiness restores grants to the current user and remembered
child realms. The lab desktop retains its bounded synchronous setup/replacement.
A failed replacement logs its actual error code before leaving storage offline.
Apps retain their tasks and drafts; Notes reports failed saves and keeps its
close dialog open when saving fails. Automatic recovery permits three provider
replacements for the session and never replenishes that budget on a healthy
reply. After exhaustion, the launcher can request one explicit save retry. There is no logout UI; recursive whole-user
LOSS remains a headless proof.

Without the `shell` feature the same package builds the probe desktop in
[lab/](lab/desktop.rs): the same admission, apps and budgets laid out as a
nested matrix beside an outside sibling, with destructive proofs selected by
feature. The proof commands the color app answers over its control link live in
[lab/commands.rs](lab/commands.rs) and never ship in the shell. Without the
`desktop` feature the package is a headless matrix fixture.


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
binding revisions, revocation during transactions, child replacement, subtrees
granted as attenuated realms, subtree copies and durable data after restarting
the tree/provider. No leaf has a disk or provider link.
See [storage scopes](../../platform/libraries/matrix/world/storage/README.md)
for the bounded realm policy and protocol.

## Visible integration

The [desktop policy](session/session.rs) supplies layout and initial appearance. The same
platform matrix implementation owns input, clipping and child admission at both
levels. The root environment additionally receives explicit compositor and
physical-source endpoints; these are the only device-facing adapters. Apps get
only their interaction pair, catalog pair and a child-local wake receiver. The [app](../applications/colors/session/session.rs) waits for local input/view invalidations and
submits an oversized rectangle that must be clipped at each ownership boundary.

The probe desktop receives code-only catalogue grants and a funded process pool.
[Startup admission](session/admission/admission.rs) stages these into the root; owner-only
RUN requests select and fund descendants using the same provider at each level.
The shell runs the chooser with RUN_OBJECT and opens slots with LAUNCH_POLICY,
so everyday programs come from the supplied realm. It explicitly lends one
boot recovery executable to the user matrix with LAUNCH_IMAGES, then selects it
as LAUNCHER_FALLBACK before requesting CONFIGURE. The user matrix may already
have configured itself from its realm; that first fallback supply preserves
the running launcher's existing retry allowance. On exhausted recovery this program
runs only in the retired strip, with no storage or child-task capacity; a local
click requests another attempt. Ordinary apps inherit no catalogue grants.
The test checks rollback, exhausted frame funding and repeated subtree replacement.
Device launches remain explicit static grants.

[Owned surfaces](../../platform/libraries/matrix/world/surface/README.md) use
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
[recovery checks](lab/recovery.rs) for revoked writes, stale bindings, termination
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
[Provider recovery](lab/providers.rs) explicitly rebuilds the root tree after
input/display loss. [Storage stalls](lab/stalls.rs) prove sibling rendering
through blocked saves and reconcile unacknowledged commits after provider replacement.
The [nested peer](lab/stalls_peer.rs) shares the saving app's immediate matrix;
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

The positional profiles fund the same shared display-configuration and root
control owners as F5. The small Colors composition reserves seven owner
channel/notification allowances (four transferred channels plus root, input
readiness and retiring display setup), two for display, two for input, two for
storage and twenty-nine for the root. Notes uses thirty-seven for its root.
The owner holds three writer pages and two readers; root funding adds one writer,
reader and notification for control without reducing the existing child/image
allowances. Provider control/fault gates and pre-driver source injection remain
explicit copied lab consumers. [source.rs](lab/source.rs) names the latter seam:
it does not construct a shared client and later hand its raw endpoints away.
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
[state entry](../applications/colors/session/state.rs) preserves confirmed values and displays a red
mark on failed saves. Enter reloads an unresolved save rather than toggling again.
The test-only [database recovery entry](lab/database_recovery.rs) sequences
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
python tools/boot-harness-rs/recovery_policy_test.py
```

The `notes` composition substitutes the separate [notes application](../applications/notes/README.md)
for the color probe and explicitly funds its retained/replacement image pages.
Its app protocol and draft state stay in that application, not in the matrix service.
