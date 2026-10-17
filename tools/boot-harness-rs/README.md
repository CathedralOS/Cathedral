# Rust boot harness

Run `python tools/boot-harness-rs/run.py --smoke` from the repository root.
This builds the separate `source-rs/` Cargo workspace and exercises its UEFI
handoff under QEMU/OVMF. Host-side Python only; no extra Python packages.

For the OS desktop in VS Code, open the repository root and press **F5** with
**Cathedral: Build & Run in QEMU** selected. There is one launch configuration;
it runs the same default composition as:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

Click **NEW** to copy the template user, click the user to run it, then click
**Notes** or **Colors** in its chrome and click the program to focus it. Notes:
type, F1 save/check, F2 reload. Colors: Enter changes color. CLOSE stops the
program unless it would lose work; an unsaved note stays until saved or reloaded.
Build output and serial diagnostics appear in the integrated terminal. Close
QEMU or press Ctrl+C there to stop. This uses VS Code's terminal launcher, not
an attached kernel debugger.

The desktop retains `build/boot-harness-rs/cathedral-storage.raw`. First launch
builds the host formatter and creates the image; later launches preserve it.
Existing files are never automatically reformatted. Older lab disks are untouched.

`python tools/boot-harness-rs/desktop_test.py` exercises this exact default image
using physical input, repeated app launches, draft protection and cold-boot saved
text checks on a disposable disk. [Distribution navigation](../../source-rs/distribution/README.md)
connects the shell, apps and platform providers.

The `notes`, `matrix-database` and recovery profiles remain focused engineering
fixtures. Their tests are `matrix_database_test.py --release --notes` and
`--release --notes-recovery`; they add isolated catalogs, exact glyph checks,
workload timing and an independent database oracle. Recovery profiles are
build-only in `run.py`.

Add `--trace-storage` for opt-in timing. Run
`python tools/boot-harness-rs/matrix_database_test.py --release --recovery` for
provider cuts, a real startup save timeout, sibling interaction, explicit grants,
exact readback and cold restoration on a disposable image. It emits timing and ATA
CSV files. See the [object store guide](../../source-rs/platform/libraries/database-objects/README.md)
and the [notes guide](../../source-rs/distribution/applications/notes/README.md)
for navigation, interpretation and remaining recovery limits. `database_image_test.py`
checks that ordinary image preparation preserves existing bytes and refuses malformed
file extents.

`python tools/boot-harness-rs/volume_test.py --release` runs the separate
[volume transport proof](../../source-rs/platform/libraries/storage-volume/CHARTER.md).
It builds [profiles/volume.json](profiles/volume.json), creates a new guarded image,
and makes four boots for byte IO/flush, cold reopen and damaged-envelope cases.
The full-image comparison and host engine reopen retain artifacts under
`build/boot-harness-rs/volume/`. Omit `--release` for the debug proof. Guest redb
execution is still pending; this exercises its volume envelope and block boundary.

`python tools/boot-harness-rs/run.py --matrix-storage-test --release` selects
[profiles/matrix-storage.json](profiles/matrix-storage.json) and checks private
catalog delegation through two matrix levels, stale binding rejection, revocation
and tree/provider replacement. The proof uses a reset private disk at
`build/boot-harness-rs/matrix-storage-test/test-storage.raw`; it never touches the
ordinary interactive disk. The guest starts at
[storage/storage.rs](../../source-rs/distribution/desktop/storage/storage.rs).

`source-rs/distribution/profile.json` selects the boot package, target and named user ELF programs. The
Cargo workspace keeps kernel implementation under `kernel/` and the UART under
`platform/drivers/`; replacing the distribution does not require moving those.

Ordinary boots build `startup.initial`, `startup.launches` and the executable
catalog. The kernel admits the initial desktop owner; `distribution/desktop`
starts providers and admits apps through recursive matrices. The kernel fixtures below are compiled
only with `--smoke`. `--kernel-only --smoke` runs kernel and raw-user tests without
building any platform service or distribution executable. `--kernel-only` boots
the kernel alone. `--profile tools/boot-harness-rs/profiles/minimal.json` supplies
only a hello initial program, with no child or display grant.

The smoke boot checks cooperative and timer-preempted tasks, dynamic admission,
task-slot exhaustion, allocation-failure rollback, stale IDs and repeated stack,
page-table and heap reclamation while a non-yielding peer stays alive. Use
`--release` for the optimized path. Expected-fault runs remain separate.
Both the x86 runner and `arm64/run.py` support kernel text/data write, heap/stack
execution, physical-alias execution and guard probes; see the
[kernel protection guide](../../source-rs/kernel/arch/memory/README.md) for commands
and exact assertions.
The composed image also checks isolated ELF programs and capability-addressed
IPC: echo exchanges, stale local handles, raw-name collisions, rights, backpressure, checked copyout,
and blocked-receiver wakeups on revoke or peer exit/fault. Each session must
return to its heap and physical-frame baselines. The userspace supervision fixture
keeps one client alive across 32 service crashes/restarts and checks explicit
reconnection, retired task/endpoint tickets, wait copyout, parent-exit cleanup,
and failed admission with live peers.
The watchdog fixture then recovers four blocked and four non-yielding services
using clock grants, deadline waits and owner-authorized cancellation. An independent
observer must progress before cancellation; an all-blocked session must also wake
on its deadline. Completed outcomes, stale cancellation tickets and complete memory
reclamation are checked as well.

The memory fixture allocates private pages, seals buffers to a boot-approved peer,
and tests RO/NX permissions, guards, stale tokens, zeroing, budgets and both peer
failure paths. It injects allocation failures at each backing/table boundary and
requires complete heap/frame reclamation. Ordinary screenshot and interactive
checks include the app's shared pixel tile. Recovery/storage builds also require
malformed pixel buffers to complete with errors and release their read leases.

The [rendering fixture](../../source-rs/distribution/lab/rendering/README.md) then
compares direct rows and flattened shared leaves on real guest pages. It checks
that a native writer can bypass a software span inside its granted page, while
an unmapped page boundary faults. Two sessions assert page-edit/IPC counts and
full reclamation. Host pixel-copy and padding measurements have a separate runner
at `tools/rendering-lab/run.py`; these experiments do not adopt a surface protocol.

The display fixture launches a platform display-service ELF and a separate
distribution client. It verifies redraw after a provider fault, unrelated task
progress, exclusive device mappings, NX/guard faults and all failed-admission
budgets. The default QEMU display device must provide a linear RGB/BGR GOP mode.

`--window` opens the QEMU display for an ordinary interactive boot; Ctrl+C
stops that run. `desktop_test.py` is the bounded scanout check of that same
composition; it needs no GUI or extra Python packages. Every capture waits for
userspace startup readiness and rejects accidental lab execution.

See [the Rust lab guide](../../source-rs/README.md) for setup, ordinary boots,
firmware overrides, logs, checks, current limitations and the bring-up sequence.
The existing `tools/boot-harness/` continues to build and boot Omega.

Smoke additionally checks a full three-child/two-link graph across 16 alternating
app/provider replacements and complete memory reclamation. The readiness fixture
checks two receive channels, first-ready precedence, no-consume observations,
denied clock/rights, peer closure and all-blocked timeout. Existing deadline,
copyout, late reply, exclusive-device and admission-failure coverage remains.

## Retained compositor profile

`python tools/boot-harness-rs/run.py --compositor-test` selects
[profiles/compositor.json](profiles/compositor.json), runs client/provider failure
probes, and independently validates all final scanout pixels. Add `--release` for
the optimized build. For a visible run use `--profile tools/boot-harness-rs/profiles/compositor.json --window`.
The [lab guide](../../source-rs/distribution/lab/compositor/README.md) describes
scope, authority, retained storage and the experimental limitations.

## Pointer interaction profile

`python tools/boot-harness-rs/run.py --pointer-test` drives the PS/2 mouse through
QMP, checks capture and app/source recovery, and compares every final scanout
pixel. Add `--release` for the optimized build. It writes
`build/boot-harness-rs/pointer-test/display.png`. See the
[pointer lab](../../source-rs/distribution/lab/compositor/POINTERS.md) for the
interactive command, gestures, protocol ownership and bounds.

## Persistent session profile

`python tools/boot-harness-rs/run.py --session-test --timeout 60` selects
[profiles/session.json](profiles/session.json). It checks event waits,
explicit input regions, delegated key masks and a separate child app clipped by
its host. It replaces the compositor during a drag, preserves the child through
host replacement, and replaces the input source while a key is held. The host
compares every scanout pixel at several stages. Add `--release` for the optimized
build. Output is `build/boot-harness-rs/session-test/display.png`; see the
[session guide](../../source-rs/distribution/lab/compositor/SESSION.md) for the
interactive command and F4â€“F7 recovery controls.

Normal session startup skips destructive bring-up probes. Add `--startup-probes`
to run those and the full-screen repaint timing probe. Session tests also drive
120 mouse reports at roughly 100 Hz and check exact motion, release and continued
keyboard input; `latency.json` records the final cursor observation time. The
interactive scene includes readable controls, focus status and recovery notices.

Add `--pressure` to `--session-test` or the interactive session profile to add a
peer that exhausts page/object grants, floods the compositor without reading
replies, and repeatedly spins/faults before supervised replacement. The same
interaction checks run under load; logs and images use the `pressure-session-test`
directory. See the [pressure lab](../../source-rs/distribution/lab/compositor/PRESSURE.md)
for navigation, checks and limits. Ordinary boots do not enable this workload.

## Recursive matrix boundary profile

`python tools/boot-harness-rs/run.py --matrix-test --timeout 60` selects
[profiles/matrix.json](profiles/matrix.json). It runs two instances of the same
matrix provider and two isolated leaf processes, checks local-only views,
clamped capture, attenuated keys, ancestor revocation, resize and replacement.
Add `--release` for optimized verification. This profile has no display grant;
its [guide](../../source-rs/distribution/desktop/README.md) documents the
execution paths and remaining environment/lifecycle work.

## CPU turn sharing and measured ceilings

The broader `--smoke` profile includes a
[CPU sharing fixture](../../source-rs/kernel/core/scheduler/README.md): one and then
three non-yielding descendants compete with an independent outer account. The
guest checks preemption shares, descendant progress, loan return and reclamation.
It also measures capped execution under single/nested spinners, competing yield
loops, repeated replacement and an all-throttled idle session. Reports include
direct/subtree microseconds and exhaustion counts, with heap/frame reclamation.

Each `startup.initial` or `startup.launches` entry may set `cpu_budget_us` to an
integer 0â€“100000. For example, 20000 supplies a 20 ms burst and continuously refills
at 20% of one CPU. Zero/default adds no local cap; ancestors still constrain the
account. Limits persist through replacement and loans. These are token-bucket
ceilings, not fixed windows or guaranteed reservations, so their sum need not fit
one CPU. [resource_budgets.py](resource_budgets.py), boot generation and kernel
admission validate the values. The 100 Hz timer permits overruns which become
debt; the QEMU-calibrated clock and unpreemptible kernel work do not establish
hard real-time bounds. See the [accounting contract](../../source-rs/kernel/core/scheduler/README.md).

## Ordinary application IPC profile

`python tools/boot-harness-rs/run.py --ipc-test --timeout 90` selects
[profiles/channels.json](profiles/channels.json). Add `--release` for optimized
binaries. The parent creates runtime channels, moves endpoints to an ordinary
worker, exchanges messages and repeats cancellation with queued transfers. It
also launches children with complete initial endpoint bundles passed to main,
checks atomic rejection, and cancels children using their initial connections.
This is headless and terminates at the guest's success marker. Logs live in
`build/boot-harness-rs/channels-test/`. The [guide](../../source-rs/distribution/lab/channels/README.md)
explains local names, backed reservations, limitations and code entrances.
The initial account has a 50 ms CPU burst with 50% refill; every worker inherits
that ancestor ceiling. This also exercises non-default profile-to-kernel CPU limits.
The sponsorship scenario retains channels across creator restarts, rejects
creation at the account limit, proves sibling capacity and recovers after closure.
The same proof now delegates launch budgets through three identical recursive
supervisors, exercises subtree cancel/exit/restart, and keeps an omitted sibling
alive. Its [admission guide](../../source-rs/distribution/lab/channels/README.md#recursive-child-admission)
distinguishes the finite boot-approved budgets from future manifest/resource policy.
[protocol_test.py](protocol_test.py) monitors both this proof and the matrix proof.

Each profile `startup.initial` or `startup.launches` entry may set `ipc_channels`
to an integer 0..20 (default 4 regardless of task count). Reservations for all
initial and dormant launch accounts must total at most 32. Charges persist
across replacement until both channel endpoints close. Launch loans preserve
the same account; they do not grant fresh capacity. [resource_budgets.py](resource_budgets.py)
validates this before executable builds; boot configuration and kernel admission
also validate the bound. Run `python tools/boot-harness-rs/resource_budgets_test.py`
for profile validation tests. Existing boot connections use a separate fixed arena.

## Integrated matrix desktop

`python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 120`
uses [profiles/matrix-desktop.json](profiles/matrix-desktop.json). The existing
compositor and physical decoder are connected explicitly to a root matrix; the
root admits an inner matrix and sibling app, and the inner admits its own app.
[The verifier](matrix_desktop_test.py) checks scanout clipping and physical focus
isolation after guest-side subtree replacement. Logs and a PNG screenshot are in
`build/boot-harness-rs/matrix-desktop-test/`.

For interaction, run `python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/matrix-desktop.json --release --window`.
Click either app and press Enter. This is the integrated matrix path; the older
session profile retains its richer compositor regression experiments. The
[implementation map](../../source-rs/STATUS.md) distinguishes their current scope.

The matrix desktop also accepts `--pressure`: a non-yielding descendant shares a
20% CPU ceiling with the nested matrix/app. Both normal and pressure tests check
quiet apps, retained placement without repaint, subtree replacement and physical
input. The harness reports key-to-scanout latency; this is not a deadline guarantee.

`--admission-test` selects `profiles/admission.json`: recursive executable-authorized
process pools, failed-load rollback, retained-account funding and sibling survival.
Add `--release` for optimized execution. Profiles can set `startup.dynamic_tasks`;
initial + static launches + dynamic accounts must fit eight. Dynamic accounts
start with zero page/channel allowances and consume explicitly assigned funding.

For opt-in input timing, add `--trace-input` to the matrix desktop command.
`INPUT_TRACE <stage> <tick> <caller_cpu_us>` records source decoding, nested matrix
receipt, app handling and completed paint. Ticks are nominally 10 ms; CPU totals
are caller-local, and logging perturbs timing. One capped-spinner comparison went
from 937 ms to 532 ms after atomic view/event polling removed redundant round
trips. The host measurement includes 160 ms of key injection sleeps and scanout
sampling; this is regression evidence, not a response-time guarantee.

Profiles with storage may list `startup.programs`: packages the harness builds,
trims to their loadable bytes and installs into the storage image as objects
one, two, three and so on of the notes root, each optionally `inside` an earlier
entry; an entry marked `folder` is an empty object for others to sit inside. A
sidecar beside the image records what was installed, so an unchanged
set never reopens the image and a changed program is republished as a new
revision on the next launch. The installer replaces only programs: an image
whose objects at those numbers hold anything else is refused rather than
overwritten. The default desktop runs every program this way;
the matrix desktop uses `startup.executables` for code-only catalogue grants and
`dynamic_tasks` for unbound accounts. Owner-only matrix RUN requests reserve and
admit each app/matrix recursively; static launches remain for the root and devices.
Its test checks failed image delegation, nested exhaustion, repeated replacement,
and outside sibling survival. Channel allowances can be up to 20 per account
(global backing remains 32 and local dynamic-handle capacity is 20), allowing
a matrix to subdivide funding while retaining its own connection capacity.

The matrix desktop now checks owned surface allocation/exhaustion, stale-handle
rejection, removal of one object, retained movement, and whole-subtree stacking.
Apps display a light accent over their base color. CPU-pressure startup also runs
repeated admission and surface lifecycle probes, so it takes longer than the normal
profile; the scene-ready timer includes those checks. Both run the same renderer.

After adding owned surfaces, recent optimized pressure runs measured 469-907 ms
(907 ms with tracing enabled); normal optimized input measured about 190-200 ms,
including the harness's 160 ms injection sleeps. Atomic polling removes redundant
round trips, but equal turns, queued input and synchronous IPC still impose visible
latency in a spinner-saturated 20% subtree. No latency guarantee is claimed.

Physical-memory checks now run with `--admission-test`: nested exhaustion, an
outside sibling's allocation, failed-load refunds and repeated subtree recovery.
`--smoke` also checks retained physical charges and every load funding boundary.
`--matrix-desktop-test` requires visible sibling rendering while nested admission
fails, before restoring the normal tree. See the
[funding contract and implementation](../../source-rs/kernel/core/user_tasks/resources/physical/README.md).

## Funded heap and guest engine

`python tools/boot-harness-rs/database_test.py [--release]` runs the
[database proof](../../source-rs/distribution/lab/database/README.md): explicit heap
funding, exhaustion and fault reclamation, an acknowledged live-engine commit,
QEMU kill, cold guest recovery and independent host verification. It creates fresh
guarded images and never uses the ordinary session disk.
