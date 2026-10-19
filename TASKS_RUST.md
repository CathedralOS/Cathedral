# Rust reference OS: tasks and handoff

Updated 2026-10-18 (local date). Current milestone: live storage-provider recovery
without restarting apps, following explicit launcher recovery
after retry exhaustion, following app termination and bounded early bootstrap recovery,
interactive storage-provider startup and explicit retry,
root login recovery, independent Colors,
local configured-launcher recovery,
scoped lifecycle observations and close-channel failure handling, live
matrix layout and named realms. F5 uses these mechanisms; explicit migration
preserves existing stock users and data.
F5 now compacts existing media before changed program installations, fixing
the retained-disk upgrade failure described at the end of this handoff.
Stock-copy refresh also collects unreachable revisions under logical quota
pressure in the same transaction as its replacements; current data and pins survive.
See the latest evidence at the end of this handoff. Earlier baseline: declared service
imports and optional Notes persistence at **`8d1972a`**, followed by readable
chooser labels at **`787344e`**, existing-user label upgrade at **`4f1c6c8`**,
app-owned close at **`a40ed23`**, user tiles/absolute input at **`4901e65`**,
the transient-picker handoff at **`30f169b`**, and the retained-disk Colors fix
below. Store-based admission validates the program's required/optional interfaces against owner
offers before spawning. Check Git for
later work.

## Start here

F5 boots one desktop to a chooser over the folder of users. **Preserve that as
the default experience.** From the repository root:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

This is exactly what the single [VS Code launch entry](.vscode/launch.json) runs.
The composition is [distribution/profile.json](source-rs/distribution/profile.json).
It uses x86-64 QEMU/TCG and UEFI.

- The full-screen user picker shows a horizontal row of avatar tiles. **NEW
  USER** copies the hidden template; click a user to enter its workspace. Up
  to three users fit the current bounded listing; the add tile then disappears.
  No scrolling, passwords or logout yet. Restart QEMU to select another user.
- The user's left-hand app strip shows **NOTES** and **COLORS** icon tiles. The
  launcher itself and implementation controls are hidden. Click an app tile,
  then the app to focus it. Arrows and Enter work in a focused tile strip.
- **SETTINGS** at the bottom of the user strip (or F2 with strip focus) adjusts
  strip width and outer spacing. Save persists and applies the layout to running
  apps; Cancel discards local edits and returns to apps. Apps retain their state.
  SAVED; RESTART means persistence succeeded but live application was refused.
  Retry Save or restart to use the saved arrangement.
- Notes keeps up to eight notes of 240 ASCII characters. Letters, digits, spaces,
  Enter, Left/Right and Backspace edit. **F1** saves/reconciles, **F2** reloads,
  **F3** duplicates the saved note, **F4** switches notes. Colors changes and
  saves its color on **Enter**.
- Notes and Colors have an upper-right **X**. Clean apps exit; dirty Notes offers
  **SAVE / DISCARD / CANCEL**. A failed save stays open. Colors asks **CLOSE
  ANYWAY** if its last storage result is uncertain. Launching another app while
  one is still open shows CLOSE APP FIRST.
- **END APP** above Settings handles an unresponsive app. It warns about unsaved
  work and defaults to Cancel; confirmation stops the exact observed app/subtree.
  F3 opens it with launcher focus. A recovered launcher can manage the existing
  app. This is delegated authority, not a protected system-wide escape key.
- If the launcher exhausts its retry, **LAUNCHER UNAVAILABLE / TRY AGAIN** takes
  its place. A click or Enter with that screen focused requests one new attempt,
  preserving the app and current arrangement. Failure of that screen stays stopped.
- The default input service negotiates QEMU's absolute virtual mouse. The guest
  cursor follows the host pointer within the window; PS/2 remains the fallback.
  Apps still receive only local matrix input, never the physical device grant.
- The old STORAGE ON/OFF toggle and launcher CLOSE row are gone. Required/
  optional service binding and the matrix LOSS/CLOSE mechanism still exist;
  their engineering controls do not belong in the normal desktop.

Interactive state persists in **`build/boot-harness-rs/cathedral-storage.raw`**:
the users you made, their notes and colors. Missing media is created; existing
media is never automatically reformatted. The programs are installed into the
image on first launch and republished as new revisions when their bytes
change; the sidecar `cathedral-storage.raw.programs` records what is installed.
The desktop's explicit compatibility upgrade refreshes recognized stock
launchers, Notes/Colors and recognized stock matrix executables in existing
users, preserving their saved data and object identities. The app-close upgrade saves
`cathedral-storage.raw.before-app-close.raw` before changing user copies; the
earlier label-only backup remains separate. Customized/unrecognized programs
stay intact. Hierarchy migration then adds names, media types and a `Matrix`
configuration, preserving existing metadata/configuration and saving
`cathedral-storage.raw.before-hierarchy.raw` before its transaction. The fixed
4 MiB lab volume can exhaust physical engine space during repeated program
upgrades; installation refuses rather than resizing or reformatting it.
An image from before this layout holds other objects at the numbers the
programs need; the installer refuses to overwrite anything that is not a
program, so move that image and its `.programs` sidecar (if present) to backup
paths before relaunching to create a fresh desktop. The backup retains the old
data; it is not migrated into the new desktop. This refusal happens before QEMU
opens, so check the launch terminal if F5 appears to do nothing. `notes-storage.raw` and
`matrix-database-storage.raw` belong to the notes and probe desktop fixtures.
Automated tests use separate disposable images.

## The architecture, in the owner's words

Realms are the attenuated filesystem. Matrices are programs that attenuate,
heavily or not. If the boot matrix is correct, holds every capability, can
launch programs, and the filesystem is database-backed and attenuable, the rest
of the matrix falls out for almost free. The real logic is then that a launched
program gets its own subtree for free: everything nested under the program's
object, because files are folders in this database.

The current bounded shape boots a startup owner that runs the providers, root
matrix and user picker; a selected user matrix loads its installed launcher.
The chooser executable serves both picker and launcher as separate processes.
This demonstrates composition and attenuation, not authenticated login or the
full windower design.

## What the store milestones established

The thesis is that cheap file copies make cheap world copies. A root holds
numbered objects arranged as a tree; chunks and manifests are shared within the
root; a copy writes a head, an event and an edge; a subtree copy does that for
every object beneath the source. A matrix grants a child a subtree rather than
a root, stamps that scope on every request it forwards, and the provider
enforces every stamp, so a program sees its realm and nothing beside it.

Programs live in the store. The harness installs the profile's `programs` into
the notes root as objects, trimmed to their loadable bytes, nested as the
profile says, with `folder` entries as empty containers. A matrix runs one with
`RUN_OBJECT`, and now a child runs one with `LAUNCH`: the matrix reads the
object through the requester's own binding, so a program can only run what it
can see, admits the sealed reply as the sibling's code in the slot the owner
opened with a standing `LAUNCH_POLICY` and `LAUNCH_PLACE`, and grants it that
object's subtree. `CLOSE` asks the host to stop the sibling; the host asks the
sibling `LOSS` on its control link first and keeps it on any answer but
nothing. Each matrix remembers its own child storage grants; explicit revocation
and child death clear that policy. After provider replacement the owner restores
the direct user's realm and asks it to restore its own children.

A user is a subtree copy of the template user: a matrix executable with the
chrome, Notes, Colors and a saved `Matrix` configuration inside it. The user
matrix loads its own launcher path, placement and budgets within the outer
owner's ceiling. `desktop_test.py` drives this from F5
and an independent host model verifies the whole layout; the headless storage
fixture runs a program from its realm, launches a sibling and closes it.

The kernel's contribution stays one syscall for code from sealed objects. It
now allows eight reader rights per task and a reader page budget of up to four
times the per-object cap, because a matrix hub pins every image its subtree
shows, holds old and new during replacement, and lends the same budget to each
nested matrix. The kernel still knows no matrix, realm or store.

## Decisions and working expectations

- Cathedral is one monorepo: an independently usable **kernel**, a blessed common
  **platform** outside it, and one replaceable **distribution**. Desktop policy
  belongs to the distribution; capability enforcement belongs to its owning
  kernel/platform mechanism. Contracts live with the boundary owner.
- A matrix is an ordinary program. The instance handed provider links is the
  storage root and the one handed display/input endpoints adapts hardware;
  every other instance only forwards and attenuates. Nothing branches on depth
  or on an argument word, and nothing tunnels through a matrix to use its
  authority: its owner grants, its children request, the provider decides.
- Delegation narrows and never widens. A grant names a scope inside what the
  granting matrix holds; the provider, not the matrices, is what makes the chain
  binding. Hosting is copying: a matrix that hosts programs holds them in its
  own realm, and a child launches only what its own binding can read.
- Use the design's vocabulary. A realm is a subtree held through a binding;
  a world is a matrix's realm in the wiki's sense (the object a matrix owns,
  with its state beneath it), which the Rust OS does not yet implement. Do not
  coin desktop-level names for storage structure.
- Boot launches and storage-less fixtures keep the boot catalogue, because the
  provider itself must come from boot. The matrix's RUN by catalogue image
  serves those compositions; RUN_OBJECT and LAUNCH serve the store.
- Timeouts derive from one number. The matrix contract's `STORAGE_DEADLINE_TICKS`
  bounds a forwarded storage request; nested hops and program reads add a
  margin, the runtime's storage client waits twice it, and the desktop's
  provider watchdog probes past it, so a stalled provider is always reported by
  the matrix's verdict first. These numbers are **ticks**: the current timer
  runs nominally at 100 Hz, so the base deadline is **six seconds**, not 600 ms.
  Keep units explicit and measure actual spans before tuning deadlines.
- Do not add debt to bridge old and new: when a protocol or layer is replaced,
  retire the old one from every composition in the same change and delete it.
  Git history is the only retention wanted.
- Discoverability matters: a named entry shows sequencing/dispatch and leads
  into semantic subfolders. Rust `mod.rs` is plumbing only, beside the substantive
  named entrance. Avoid forwarding-only layers and unrelated folder dumping grounds.
- Keep the OS responsive and understandable. Rendering/input changes need an
  actual QEMU interaction check, not just compilation or a serial success marker.
- Commit and push progress to `main` after each session. Check the working
  tree first and preserve unrelated work. `.codex/` is untracked and
  deliberately untouched.
- Record unresolved owner-level semantics/trust choices in
  [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md). Ordinary implementation bugs and
  refactoring do not require an owner question. Existing design prose contains
  ambitions and unresolved conflicts; do not treat it all as implemented fact.

## What exists and where to enter

| Area | Current implementation | Reading entrance |
| --- | --- | --- |
| Kernel | UEFI exit, owned mappings/stacks, frame/heap allocation, interrupts/timer, preemption, isolated user processes, capability syscalls, IPC, admission/teardown from boot templates or sealed objects, recursive resource funding and CPU ceilings; 128 private and up to 512 reader pages per task, eight reader rights | [Boot](source-rs/kernel/boot/uefi/main.rs), [user tasks](source-rs/kernel/core/user_tasks/user_tasks.rs), [code grants](source-rs/kernel/core/user_tasks/admission/pool/README.md) |
| Architecture | Shared core with x86-64 and an ARM64 QEMU portability proof; ISA and board responsibilities separated | [Architecture](source-rs/kernel/arch/README.md), [ARM64 bounds](source-rs/kernel/arch/isa/aarch64/README.md) |
| Matrix | Same service recursively owns children, local surfaces, focus, input, realm delegation with scope stamps and attachments, programs run from its realm by owner or child, standing launch policy per slot, close-after-asking, bounded budgets and deadlines; counts unanswered child requests instead of killing a slow child | [Service loop](source-rs/platform/services/matrix/service/service.rs), [protocol](source-rs/platform/contracts/matrix.rs), [child launches](source-rs/platform/services/matrix/service/children/children.rs), [storage forwarding](source-rs/platform/services/matrix/service/parent/storage/storage.rs) |
| Display/input | Software compositor, retained rectangles and sealed bitmap objects, scoped pointer/focus routing, physical PS/2 input | [Display](source-rs/platform/services/display/main.rs), [input](source-rs/platform/services/input/main.rs) |
| Storage | Userspace ATA PIO, guarded redb volume, objects up to 256 KiB, shared content, subtree copies, names/types on edges, scoped relative lookup and indexed type filtering, pins and bounded collection | [Provider](source-rs/platform/services/storage/main.rs), [store](source-rs/platform/libraries/database-objects/README.md), [contract](source-rs/platform/contracts/storage.rs) |
| Desktop | Provider admission/supervision, named login paths, chooser over users, user ceiling and startup request; each user matrix loads its own arrangement. Probe desktop under `lab/` uses the boot catalogue without the shell feature | [Main](source-rs/distribution/desktop/main.rs), [session](source-rs/distribution/desktop/session/session.rs), [shell](source-rs/distribution/desktop/session/shell/shell.rs), [lab](source-rs/distribution/desktop/lab/desktop.rs) |
| Chooser | Lists its realm, launches an entry as its sibling, copies the first entry on NEW, asks the host to close | [Chooser guide](source-rs/distribution/applications/chooser/README.md), [session](source-rs/distribution/applications/chooser/session/session.rs) |
| Notes | Whole-document publication in the binding's realm, bulk reads, duplicate through the store's copy, note switching, explicit reconciliation, sealed bitmap presentation, answers the close question with its draft state | [Session](source-rs/distribution/applications/notes/session/session.rs), [storage](source-rs/distribution/applications/notes/session/storage.rs) |
| Harness | Builds, trims and installs programs and folders; formats and seeds images; host oracles over copies of guest images | [Harness guide](tools/boot-harness-rs/README.md), [seeder](tools/storage-lab/engines/volume/seed.rs) |

The [distribution map](source-rs/distribution/README.md) connects executable and
IPC seams. [STATUS.md](source-rs/STATUS.md) has broader mechanism/proof coverage;
[the Rust guide](source-rs/README.md) has ABI/build details.

The kernel is still bounded: twelve task accounts, a small boot-supplied code
catalogue for boot launches, one CPU, static device grants and finite
page/channel budgets. ARM64 is a headless portability proof, not the desktop.
There is no production DMA/IOMMU driver stack, SMP, GPU stack, audio stack or
production DBFS. Do not equate the object store or BLAKE3 layout research with
a finalized filesystem format.

## Composition facts worth knowing

The owner starts display (Launch 0), input (1), root matrix (2) and object
storage (3) from boot. The programs are objects 1 to 6 of the notes root: the
chooser; the folder of users; inside it the template user, a matrix
executable; and inside the template its chrome (the chooser again), Notes and
Colors. Identical bytes share content. These numbers are installer locations,
not runtime naming policy: [shell.rs](source-rs/distribution/desktop/session/shell/shell.rs)
resolves `Login` and `Users`; the picker resolves `Template`; each user matrix
reads its `Matrix` configuration and resolves its launcher path. Roles come from each executable's entry: a matrix
takes 0, the desktop executable's 2 runs Colors with argument 2, Notes and the
chooser take 2.

Root slot 0 holds the chooser, shown only the users folder; root slot 1 is
opened by a standing policy and holds whatever the chooser launches, a user
matrix. The desktop notices it by polling `STORAGE_BINDING` on that slot and
requests `CONFIGURE` after collecting the picker. The user loads its own chrome
into slot 0 over its realm and opens slot 1 for an app using its saved policy.
NEW copies the template's executable, programs, metadata and configuration;
copied path strings stay unchanged. Apps do not know global coordinates or
siblings. Storage replacement restores the direct user's binding, then requests
`RESTORE_STORAGE`; the matrix retains responsibility for its children's grants.

Page budgets are sized for sealed bitmaps held at every matrix level with old
and new present during replacement, plus the largest program a matrix reads
to run it: the root matrix has 128 private and 256 reader pages and lends the
user matrix 80/96; the chooser has 48 private pages (three 12-page tiles
and a replacement); Notes 32/4; Colors fits in the same; the provider 128/64; the
display 128 reader pages. The kernel's per-task bound is 128 private and 512
reader pages with eight reader rights; the kernel heap is 256 KiB. Objects
hold at most 256 KiB (64 pages).

A request carries three scope words, so three narrowing grants can stack on one
path; a fourth is refused. Each scoped request costs the provider one ancestry
walk per stamp. Collection before every write scans only metadata and the
key-only chunk index. The older 50/100/150-ms collection/commit/save claims were
not reliable unit conversions and are withdrawn. A fresh traced desktop run is
recorded below. The matrix's base storage deadline is 600 **ticks**, nominally
six seconds; the runtime's 1200-tick wait is nominally twelve. Values were not
retuned in the recursive-close change. All are lab bounds.

`--smoke` uses the default profile's `user_programs`, not its desktop startup
graph; the desktop is tested by `desktop_test.py`.

## Next work, in recommended order

The accepted [matrix ownership contract](wiki/spec/execution/components_and_matrices.md)
now resolves the startup/persisted-policy owner question. A matrix is an ordinary
program whose parent grants its ceiling; the matrix controls its own children
within it, using code, configuration, or both. The privileged startup environment
need not be one matrix executable. The
[storage contract](wiki/spec/storage/realms_and_objects.md) requires independent
copies and unchanged path references resolved within the supplied view. Stable
persisted IDs and aliases are deferred. Names and queryable media types belong
to hierarchy metadata; extensions are conventions, not authoritative types.

- [x] **Hierarchy names and queryable media types.** Metadata and indexes commit
  atomically with revisions; lookup and exact-type filtering obey subtree scopes.
  The chooser uses names/types and resolves its hidden template/launcher by path.
  Existing stock users migrate without renumbering or replacing their saved data.
- [x] **Make the user matrix own its startup arrangement.** The stock matrix
  loads `Matrix` in its realm and resolves its saved launcher path. Startup
  requires empty slots; missing paths return to the picker. Parent grants remain
  the ceiling. Custom matrix programs need not use this bounded schema.
- [x] **Make saved layout editable through the desktop.** The launcher edits
  strip width and outer spacing using its existing user-realm grant. Expected
  revisions prevent stale overwrites; lost replies require reconciliation.
  Initial defaults now live in distribution/matrix.conf. Budgets and launcher
  selection are not exposed in the UI.
- [x] **Explicit live layout application.** Stock configuration grants its
  selected launcher geometry authority tied to that endpoint incarnation.
  The child request validates both rectangles, updates future app placement,
  and never changes budgets, services or key masks. Apps recover stale drawing
  handles while retaining documents; tests force two rendering races.
- [x] **Recover refused/crashed app launches and failed user startup.** The
  launcher observes only its latest launched task, reports CANNOT OPEN or APP
  STOPPED, and stays usable. User CONFIGURE failure returns to the picker even
  if collection has already won the race. Placement failure rolls admission back.
  Broken control channels alone cannot authorize close or prove process death;
  task outcomes distinguish those cases. Tests exercise both control directions,
  owner-stop races and exit during a loss survey.
- [x] **Recover the configured launcher locally.** One automatic retry preserves
  the running app, its draft and current layout. Fresh endpoints/bindings carry
  fresh authority; ancestor changes, owner revocation and STOP cancel recovery.
  A repeated failure stops the loop, and a clean exit stays closed.
- [x] **Make login and storage-startup failure visible and retryable.** Missing
  Login/Users paths, picker failure and failed storage readiness show a bundled
  recovery screen. Storage opens behind live input/rendering; a fresh user action
  authorizes each retry. Retries neither repair media nor replenish the automatic
  provider replacement budget.
- [x] **Bound and unwind early bootstrap failures.** Initial display/input/root
  admission gets one retry after task/endpoint cleanup and conservation checks.
  Exhausted root/input recovery shows a static restart screen; failed display
  leaves an explicit launch-terminal diagnostic. Storage has not started yet.
- [x] **Explicitly end an unresponsive app.** The configured launcher receives
  management authority bound to its endpoint and the app slot. END APP warns
  about unsaved work, defaults to Cancel and names the exact observed task;
  stale/revoked tickets cannot kill replacements. A recovered launcher receives
  fresh authority over the existing app. Apps and storage-only endpoints deny it.
- [x] **Offer explicit recovery after launcher retry exhaustion.** Owner-supplied
  fallback code occupies only the retired launcher view. Its current connection
  may request one new attempt; no automatic budget is restored. App drafts survive.
- [x] **Exercise live storage loss in the F5 desktop.** Replacement readiness is
  asynchronous; fresh grants follow readiness, without restarting Notes or login.
  Precommit/lost-reply faults, exhausted retries and blocked replacement readiness
  exercise draft retention, reconciliation and offline close protection.
- [ ] **Continue lifecycle recovery where it is still incomplete.** Explicit
  live-session storage retry, display/input/root loss and clearer focus feedback
  remain. A live storage retry needs delegated authority and a discoverable
  control without replacing the user's app or expanding an app's service grants.
  Existing app-owned close protects dirty work, and layout invalidation keeps drafts.
- [ ] **Bring storage write latency down.** Collection scans metadata before
  each write and durable commit writes through ATA PIO. Use explicit tick-to-time
  conversion and separate CPU time from elapsed time. Measure with
  `--trace-storage` before changing either; the small sample below is not a
  benchmark or a latency guarantee.
- [ ] **Let the lab desktops run from the store too.** The probe desktop and
  the notes fixtures still lend the boot catalogue, which keeps the admission
  and funding proofs on their original path. When those proofs have store
  equivalents, retire RUN by catalogue image from the matrix.
- [ ] **Choose the next kernel/device milestone deliberately.** A second
  userspace MMIO/IRQ device proof is the candidate. DMA needs a separate buffer
  ownership/isolation design.

An intermittent ATA timeout is recorded in the implementation map and remains
undiagnosed. Retain logs and distinguish a reproduced transport failure from a
guessed cause if it recurs. Once in nine runs, `matrix_database_test.py
--notes-recovery` saw the provider's quiet post-commit gate let the first save
through instead of stalling it; the provider now traces `gate-armed` and
`gate-stall` under `--trace-storage` so the next occurrence shows whether the
gate was armed on the instance that served the save.

## Validation and last evidence

Run Cargo commands **inside `source-rs/`** so its toolchain, aliases and linker
configuration apply:

```powershell
cargo test --locked
cargo test --locked -p cathedral-database-objects --features host
cargo check-userspace
cargo check-uefi
cargo clippy --locked --package cathedral-desktop --target x86_64-unknown-none --features shell -- -D warnings
```

From the repository root, select checks appropriate to the change:

```powershell
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 180
python tools/boot-harness-rs/run.py --matrix-desktop-test --pressure --release --timeout 240
python tools/boot-harness-rs/run.py --matrix-storage-test --release --timeout 120
python tools/boot-harness-rs/matrix_database_test.py --release
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
python tools/boot-harness-rs/matrix_database_test.py --release --notes
python tools/boot-harness-rs/matrix_database_test.py --release --notes-recovery
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/resource_budgets_test.py
python tools/source-layout/check.py
python tools/source-layout/check_test.py
python tools/source-layout/format-rust.py --check
```

`desktop_test.py` builds the **same default composition as F5**, installs the
same programs into a disposable image, copies a user from the template with
physical mouse input, launches its programs from the chrome, compares exact
note glyphs, status and note bar, refuses dirty closes, saves, duplicates,
edits the copy, switches back, reloads, repeatedly replaces apps, checks Colors
keeps its color, cold-boots the same disposable disk, and has the host lab
verify the retained records on a copy of the image. `--trace-storage` adds
provider phase timestamps to its serial log. Kernel/arch changes need the
smoke and ARM64 suites documented in their guides:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
python tools/boot-harness-rs/arm64/run.py --release
```

Passed before this handoff, on the chooser commit: the five Cargo commands
above plus clippy of every desktop, notes, matrix and storage feature set the
profiles use, both layout checks and formatting, every harness command listed
above (the notes recovery fixture eight times in a row after the one failure
noted above), the x86 release smoke suite, the ARM64 release proof,
`--matrix-test`, `--admission-test` and `--ipc-test`. Oracles verified the F5 image (six
installed objects, a user copied from the template, two notes sharing one
manifest under its Notes, the color under its Colors, empty second root), the
probe desktop images, the notes images, and both objects proofs.

When handing off again, update the baseline commit, next concrete task, known
failures and verification scope here. Keep durable subsystem detail in its owner
docs, and avoid leaving two conflicting descriptions of the default OS boot.

## Recursive-close handoff evidence

The matrix's [loss survey](source-rs/platform/services/matrix/service/children/loss.rs)
asks both children without blocking their upstream IPC. It replies only after
all requested answers settle and returns only a boolean aggregate, never child
identities or draft contents. Dirty, missing, busy and failed answers keep the
matrix open. Admission/topology commands refuse while a survey is active.
LOSS remains an advisory observation of cooperative children, not a distributed
freeze that prevents later edits. General crash recovery and persisted matrix
configuration remain separate work.

The headless `--matrix-test --release` now checks dirty descendants, a deliberately
delayed clean reply followed by dirty replies, subtree survival after refusal,
exhaustion and restoration of the eight-unanswered-probe allowance without
closing the live child's endpoint, and clean subtree close/recreation while an
outside sibling survives. At that baseline, the actual
F5 test checked the outer CLOSE against a dirty note, repeated whole-user close and
reopen with clean Notes still running, closed a user containing only chrome, and
verified persistent Notes/Colors plus the cold-boot host oracle. The phase change
below removes those outer controls from the ordinary desktop.

Fresh traced F5 evidence: `build/boot-harness-rs/desktop/20261017T231701527969Z/`.
The first boot had four measured publication spans and two note saves:

| Span | Count | Min / median / max ticks | Nominal milliseconds (10 ms/tick) |
| --- | --- | --- | --- |
| Collection | 4 | 14 / 57 / 73 | 140 / 570 / 730 |
| Commit | 4 | 7 / 21 / 122 | 70 / 210 / 1220 |
| Whole note save | 2 | 80 / 129 / 178 | 800 / 1290 / 1780 |

These spans include normal traced publication work, not injected stalls. They
come from one small TCG run on a host also doing validation work, and do not
establish a stable performance distribution. The deadline-unit correction
changes documentation and the constant's name, not the timeout values.

Validation for this recursive-close change: `cargo test --locked`,
`cargo check-userspace`, shell/matrix target clippy, layout checks, formatting,
`--matrix-test --release` (including exhausted-probe recovery),
`--matrix-storage-test --release`, `matrix_database_test.py --release
--notes-recovery`, and the F5 desktop test with and without storage tracing passed.
The final ordinary desktop run is
`build/boot-harness-rs/desktop/20261017T232223657195Z/`; its independent oracle
verified all 189 retained records after cold boot. Notes recovery evidence is
`build/boot-harness-rs/matrix-database/20261017T232129731671Z/` (16 records).
The earlier intermittent gate miss did not occur in this run; it remains open.
No kernel code or timeout value changed, and no new smoke/ARM64 run is claimed.


## Desktop phases after the recursive-close baseline

The ordinary desktop now centers the user picker, then revokes that chooser's
surface/input grant when a user is furnished. The boot matrix still hosts and
supervises the user, but its picker and global help no longer occupy the screen.
The user receives the full display: a launcher down the left and one app in the
remaining area. No logout control is exposed. Restart QEMU to select another
user; the headless matrix fixture remains the whole-user LOSS/teardown proof.

Follow `shell/shell.rs` for admission and `shell/layout.rs` for phase placement.
The chooser renders its own heading (entry argument 1: CHOOSE USER; otherwise
PROGRAMS), so it retires with its ordinary surfaces. Its two images now require
40 private pages; the user receives 80 to fund that chooser, Notes and spare
capacity within the existing root budget. No kernel or platform protocol changed.

The F5 test now checks picker removal, clicks the former boot CLOSE position
with both a dirty and a clean note open, checks the app reaches the expanded
viewport, replaces apps repeatedly, and cold-boots the same disk to verify
saved text/color and the independent host oracle. Its previous interactive
whole-user close/reopen steps are intentionally superseded by this phase flow.

Validation: `cargo test --locked`, `cargo check-userspace`, shell/chooser target
clippy with warnings denied, layout and formatting checks, and the updated
`desktop_test.py` passed. Final desktop evidence:
`build/boot-harness-rs/desktop/20261018T033411184475Z/` (189 retained records
verified after cold boot). `user-picker.png` and `user-workspace.png` show the
phase transition. No new kernel smoke, ARM64 or headless matrix run is claimed
for this distribution-only change.

## Declared imports and optional persistence handoff

Store programs now carry a bounded required/optional service declaration in
their sealed ELF. Start at [the import contract](source-rs/platform/contracts/imports/README.md)
for parsing, version selection and the admission/runtime reading path. The
matrix resolves it against the owner's offer before reserving or spawning.
Children may withhold offered services but cannot widen that offer. Missing or
incompatible required imports refuse launch; optional storage becomes an actual
`Option` in the runtime view, with no storage endpoints or subtree grant.

F5's user launcher now toggles STORAGE ON/OFF for subsequent launches. Notes
accepts absence and labels its viewport TEMP NO SAVE. Save and reload refuse
without discarding the draft, and dirty-close protection still applies. Colors
requires storage and refuses with NEEDS SERVICE. Re-enable storage and relaunch
Notes to recover saved data. Use NEW at the user picker for current copied app
binaries; existing user copies retain their installed code. Undeclared legacy
images explicitly require both services, so they still run with storage and
refuse when it is withheld.

This proves two wire contracts and the existing three/five-endpoint startup
shape. It does not implement arbitrary endpoint tables, global discovery,
first-request activation or a final persisted manifest. Boot/catalogue fixtures
retain explicit wiring. The startup architecture notes now distinguish
privileged service composition from the matrix executable and hosted login UI;
persisted policy ownership and object kinds/names remain owner questions.

Validation: `cargo test --locked`, the contract tests, `cargo check-userspace`,
shell/chooser/Notes/matrix target clippy with warnings denied, layout and format
checks passed. The updated `--matrix-storage-test --release` passed owner-offer
attenuation, refused widening, refusal/retry, confinement and revocation checks.
The F5 desktop test passed temporary-mode rendering, save/reload refusal, dirty
close, storage restoration, repeated replacement and cold boot; its independent
oracle verified 189 retained records with no temporary writes. Final evidence:
`build/boot-harness-rs/desktop/20261018T052214126134Z/`, including
`temporary-notes.png`.

`matrix_database_test.py --release --notes-recovery` also passed, with 16 records
verified after each cold boot. Evidence:
`build/boot-harness-rs/matrix-database/20261018T052026510579Z/`. The earlier gate
miss did not recur and remains open. During implementation, Rust's `#[used]`
emitted GNU-retain ELF metadata that the kernel correctly refused; the final
declaration uses an exported symbol plus linker KEEP, preserving the accepted
ELF ABI without relaxing kernel validation. No kernel code changed; no new
kernel smoke or ARM64 run is claimed for this milestone.

## Readable chooser labels

The picker now shows USER TEMPLATE, USER 1, USER 2, etc., and NEW USER. The
program launcher shows LAUNCHER, NOTES and COLORS. Profile `display_title`
values are installed as a small footer in program objects, retained by subtree
copies and read through the chooser's own scope. No kernel, platform protocol
or database layout changed. These are display labels; user ordinals are not
editable account names. General object naming/kinds remain open.

At this milestone existing users still retained their old launcher; the upgrade
below corrects that omission. Missing or invalid program titles use PROGRAM 1,
PROGRAM 2, etc.
See the [chooser guide](source-rs/distribution/applications/chooser/README.md)
for the bounded format and reading path.

Verification: chooser target clippy with warnings denied, layout/format and
diff checks passed. The F5 desktop test checked exact labels before and after
cold boot, optional storage, draft protection and persistent state; the host
oracle verified all 189 retained records. Evidence:
`build/boot-harness-rs/desktop/20261018T053452234078Z/`, including
`user-picker.png` and `user-workspace.png`. No new kernel/ARM64 run was needed.

## Existing-user label upgrade

The first label fix only refreshed installed templates, leaving existing users
with OBJECT rows. The [distribution migration](source-rs/distribution/migrations/README.md)
now identifies old stock copies by retained template manifests or the verified
legacy allowlist. It replaces their launcher and appends labels to old app code
in place, preserving object identities, saved data and customized programs.
F5 runs it after seeding and records its fingerprint in the install sidecar.
The general policy for pinned copies versus managed program updates remains
undecided; this is an explicit distribution compatibility upgrade.

The interactive disk had two users with old stock code. Six program objects
were upgraded, with a backup at
`build/boot-harness-rs/cathedral-storage.raw.before-readable-labels.raw`. No
new user is required. Old Notes/Colors retain their previous features.

Verification: host migration tests and storage-lab clippy with warnings denied
passed. Compatibility replay of the pre-label desktop fixture entered its
existing user, checked the exact labels and read both saved notes:
`build/boot-harness-rs/labels-upgrade/20261018T163807055999Z/`.
The normal F5 desktop and cold-boot proof also passed, with all 189 records
verified by the host oracle:
`build/boot-harness-rs/desktop/20261018T163829248414Z/`.
No guest code, kernel protocol or database schema changed in this correction.
Layout/format checks and the persistent-image host tests passed too. The latter
exposed an existing eager host-tool build before the no-op/invalid-image checks;
image preparation now builds only when formatting, seeding or upgrading is needed.

A booted copy of the actual interactive disk confirmed USER 2's corrected labels:
`build/boot-harness-rs/labels-upgrade/interactive-check-20261018T164544/`.
The preceding two attempts hit the already tracked ATA completion timeout before
UI startup: op=30, LBA=24, status=d0, 50 ticks, with 4825 and 5347 polls. Their
logs remain under `interactive-copy-20261018T164207/` and
`interactive-retry-20261018T164335/` in the same labels-upgrade directory. The
next attempt passed without changing the driver or deadline; the cause remains
unresolved. The interactive source image was never attached to those guest tests.


## App-owned close controls

Notes and Colors now render an X in their own local viewport. Closing returns
from the app; the existing matrix child-disconnection path reclaims its grants,
surfaces, input and launch slot. There is no new close syscall or matrix opcode.
Notes presents SAVE / DISCARD / CANCEL when work is unconfirmed; temporary
Notes disables save. Cancel preserves the draft and modal keyboard input cannot
edit it. An uncertain save reconciles first; failure or a differing confirmed
value keeps the dialog open for an explicit next action. Colors requires CLOSE
ANYWAY after a storage error, and its LOSS reply now reports that error too.

Follow Notes' session into `closing.rs` for pointer actions and `view.rs` for
presentation; Colors' app entrance similarly descends into `closing.rs`.
Notes uses its existing text bitmap for the dialog and its sixth allowed surface
for the X. An initial separate dialog bitmap exceeded the 32-page allowance
when replacing text; reusing the text bitmap fixed that without raising budgets.

The profile explicitly opts into `app_close_migration`. Host installation
refreshes only recognized stock Notes/Colors and launchers, including the old
apps previously given title-only footers. Recognition checks content digests;
a title alone cannot authorize replacement. User matrix code, object identities,
child data and customized programs remain unchanged. The actual interactive
image's two users received four app updates, backed up at
`build/boot-harness-rs/cathedral-storage.raw.before-app-close.raw`.

Verification for this change:

- Notes host test, shell Notes/desktop and notes-probes target clippy with
  warnings denied, four host migration tests, storage-lab clippy, two persistent
  image tests, layout/format and diff checks passed.
- Default F5 desktop proof covered clean exit/relaunch, dirty save/discard/cancel,
  modal draft protection, temporary refusal/discard, launcher LOSS behavior,
  saved colors and cold reboot. Host oracle verified 193 records. Artifacts:
  `build/boot-harness-rs/desktop/20261018T172431571623Z/`.
- Existing-user compatibility replay upgraded the pre-label fixture, checked
  both saved notes, and used the X to close/relaunch inside its old matrix:
  `build/boot-harness-rs/labels-upgrade/20261018T172626738714Z/`.
- Notes recovery/focus/glyph checks and cold restoration passed, with exact
  16-record host oracles:
  `build/boot-harness-rs/matrix-database/20261018T172850295341Z/`.

This does not supply owner force-close for an unresponsive app, a global escape
shortcut, or new focus indicators. Colors' uncertain-close confirmation was
compiled but not separately fault-injected through the shell UI. Kernel and
ARM64 suites were not rerun for this application/installer change. The existing
intermittent ATA timeout remains unresolved; none appeared in these runs.


## User tiles and absolute pointer

The ordinary desktop now opens to horizontal user/avatar tiles across a
full-screen picker, with the template hidden. NEW USER makes a copy; at the
existing three-user listing limit the add tile disappears. No scrolling or
account-management framework was added. Inside a user the same chooser shows
Notes/Colors icon tiles. The self-launcher entry, STORAGE ON/OFF, launcher CLOSE
and normal status rows are gone. Failed actions give brief text; apps retain
their X buttons and close decisions. Required/optional imports and matrix
LOSS/CLOSE remain platform mechanisms, without default GUI test controls.

Chooser sequencing stays in `session/session.rs`; `layout.rs` shares geometry
with pointer hit testing, `view.rs` publishes surfaces and `tiles.rs` owns raster
icons. Five chooser surfaces plus six app surfaces fit the user matrix's eleven;
the root's five picker surfaces and eleven user surfaces fit its sixteen.
The chooser receives 48 private pages. Each tile uses 12; old failure text is
released before replacements to keep three-user repaint within that budget.

The default input profile enables `absolute-pointer`. The input service's
`absolute.rs` negotiates QEMU VMMouse and scales absolute axes; the matrix API
still gets ordinary local coordinates. A restricted INPUT_VIRTUAL_POINTER syscall
requires the exclusive bootstrap input grant, validates the complete reply
buffer before any device effect, and delegates whitelisted operations to the
PC board adapter. No arbitrary ports or general hypervisor calls are exposed.
The owner retirement path disables virtual input; unavailable/failed absolute
input falls back to PS/2. Other lab profiles keep relative input. This is QEMU
integration, not a USB/HID driver. See `platform/services/input/README.md`.

The existing stock-program migration refreshed both launchers and Colors in the
actual interactive disk (four copied objects). Notes data and user matrix code
remain intact. A separate pre-upgrade backup is
`build/boot-harness-rs/cathedral-storage.raw.before-user-tiles.raw`.
A booted copy showed both user avatars, entered USER 2, and opened its Notes:
`build/boot-harness-rs/labels-upgrade/interactive-tiles-20261018T174902/`.

Verification:

- Full Rust host suite, chooser/desktop/input and kernel target clippy,
  layout/format/diff checks passed. The new board test rejects non-pointer
  commands and oversized/truncated operands. Colors checks that an ordinary
  app cannot access the physical virtual-pointer transport.
- Default F5 desktop passed absolute-coordinate clicks, user creation, app
  launching, dirty save/discard/cancel, repeated exit/relaunch and cold reboot;
  host oracle verified 199 records:
  `build/boot-harness-rs/desktop/20261018T174221890406Z/`.
- The same desktop flow with VM port disabled passed PS/2 fallback:
  `build/boot-harness-rs/desktop/20261018T174441789342Z/`.
  Reproduce with `desktop_test.py --relative-pointer`.
- Existing-user upgrade, saved notes and app close passed on the pre-label
  fixture: `build/boot-harness-rs/labels-upgrade/20261018T174406660935Z/`.
  Three-user creation, full-row keyboard repaint, existing notes and close also
  passed on a newer fixture at `labels-upgrade/20261018T174733693334Z/`.
  Use `labels_upgrade_test.py IMAGE --fill-users` for that additional limit check.
- Release x86 smoke and ARM64 portability boots passed. No new ARM device was
  introduced; the shared syscall routing still compiles/runs there.

**Earlier storage finding (retest below now passes):** With the older
`desktop/20261017T232223657195Z/guest.raw` fixture upgraded, adding USER 2 twice
hit a storage deadline and provider replacement; the picker displayed TRY AGAIN.
Existing-user selection/readback passed before that extra stress. Artifacts:
`labels-upgrade/20261018T174631398112Z/` and `20261018T174825636437Z/`.
The newer fixture's equivalent two extra copies passed. At that point the
cause was unproven. After bounding foreground collection, the exact older
fixture now creates both extra users and preserves its notes:
`labels-upgrade/20261018T181109147782Z/`. This is consistent with the traced
Colors cleanup stall below; the earlier copy failure itself was not traced,
so do not relabel it as a proven ATA fault or claim every uncertain-copy case
is fixed.
No retry of an uncertain copy was added. The original chooser already lacks a
reconcile/reload action after uncertain creation; investigate that alongside the
storage stall before adding broader account controls. The new capacity check
is opt-in so its distinct purpose is explicit.

The old F5 test's storage-toggle/temporary-Notes steps were removed with that
UI. Earlier evidence remains above; service import policy host tests still run,
but this milestone does not claim new end-to-end optional-storage UI coverage.


## Transient picker handoff

The login design already says the broker exits after root transfers the seat.
The Rust picker was instead kept alive with its display/input revoked only
**after** loading the user's launcher. Slow storage therefore left the old
picker visible during launcher startup. The chooser guide also misleadingly
said the same program "becomes" the app strip; those were always separate
process instances, now documented with the process tree in distribution/README.

`shell::furnish` now starts by STOPping and collecting root slot 0 through its
existing owner interface. That path flushes retired surfaces, removes input and
storage connections, waits for process death and returns funding. The owner
checks the old control route is PEER_CLOSED, then reads/starts the user launcher.
No kernel/matrix protocol change. The selected user remains its sibling; picker
teardown cannot kill that user's tree. If the user later disappears, the owner
runs a fresh picker instead of trying to reveal the collected one. Provider
recovery grants only the currently live side of this phase boundary.

This implements lifecycle only. Warden verification, secure credential entry,
realm unsealing and seat-bound session leases are still not implemented. The
existing design is the reference; there is no new owner question about whether
the broker should stay in the user's process ancestry.

Validation: shell target clippy, layout/format/diff checks and the full default
F5/cold-boot test passed at `desktop/20261018T175557759835Z/` (199 host-oracle
records). The test now checks the whole former avatar band, clicks the old tile
location and checks it stays clear, and checks the picker-collected milestone
precedes user-launcher readiness. The former single-pixel check sampled a gap
between tiles and could miss stale avatars. User-matrix crash recovery's fresh
picker path was updated but not fault-injected in this pass. No kernel or ARM
changes; those suites were not rerun.

Existing-user compatibility replay also passed with the stronger handoff checks:
`labels-upgrade/20261018T175748538255Z/`. The interactive image's recognized
stock Colors copies were refreshed because Colors shares the owner executable;
the actual fix runs in the booted owner, so old user matrix/launcher binaries
need no changes. Saved data is preserved; the pre-update image is at
`cathedral-storage.raw.before-picker-handoff.raw`.


## Colors on retained desktop disks

Reproduced the owner's actual failure on a copy of the two-user interactive
image. Colors' initial six-byte state publication entered collection; the
provider tried up to 64 passes (each rescanning liveness and committing eight
removals) before doing the save. Accumulated program-upgrade history exceeded
the 600-tick/six-second IPC bound. Root revoked the user's storage binding;
the owner interpreted that as user death, attempted to restart the picker,
and panicked on BUSY. The last frame then looked frozen. Before-fix evidence:
`colors-check/20261018T180419Z/`, traced `colors-check/20261018T180654Z/`.

The provider now performs one bounded pass per publication/copy. It preserves
atomic publication and exact accounting; additional maintenance proceeds on
later mutations or explicit COLLECT. It can return capacity before all garbage
is drained. Background collection and quota-pressure scheduling remain future
work. The owner separately confirms a closed control route before forgetting
a remembered realm; a storage error, BUSY or timeout cannot restart login.

`colors_test.py IMAGE --users N --user I` upgrades a disposable copy and checks
Colors **first**, save, clean close, reopen, and cold-boot saved color, rejecting
panics or provider replacement. Both actual saved users passed:
`colors/20261018T180957340154Z/` (USER 2, first launch 2.52s, cold 0.38s),
`colors/20261018T181122360912Z/` (USER 1, 3.30s / 0.73s). Times include scanout
observation overhead and are TCG samples, not performance guarantees. The same
USER 2 disk without the upgrade step opened in 1.36s after the click at
`colors-check/20261018T180812Z/`.

Full default desktop/cold boot and independent 201-record oracle passed at
`desktop/20261018T180905741067Z/`. Object retention, bounded collection, exact
accounting and three cold boots passed at `objects/20261018T181138302783Z/`.
Host `cargo test --locked`, shell/storage target clippy, formatting and layout
checks passed. Kernel/ISA behavior is unchanged; no new ARM boot claimed.


The explicit lost-reply variant (`colors_test.py IMAGE --users 2 --user 2
--recovery`) also passed, including reconciliation and cold boot, at
`colors/20261018T181550212623Z/`. It adds the existing quiet post-commit gate to
the F5 composition and checks that exactly one provider replacement occurs
without restarting the picker. This exposed a second recovery bound: opening
an unclean retained volume exceeded the old 500-tick startup wait. Replacement
startup now has a separate 3000-tick/30-second bound; individual storage IPC
operations retain their six-second bound. Replacement failures now log the
actual error code. Earlier injected failures and the successful exploratory
run are retained at `colors-check/20261018T181226Z/`, `20261018T181317Z/` and
`20261018T181411Z/`. The longer startup bound is not a recovery-speed claim.

Final ordinary (fault gate absent) replay passed at
`colors/20261018T181641027782Z/`. The normal F5 EFI was rebuilt and the actual
interactive disk's recognized Colors copies upgraded with saved state intact;
`cathedral-storage.raw.before-colors-launch.raw` preserves the disk before
this update. The existing app-close migration backup is also retained.

## Matrix policy ownership decision (2026-10-18)

The owner question on startup composition and persisted matrix policy is closed
into [components and matrices](wiki/spec/execution/components_and_matrices.md).
The parent grants the ceiling; the matrix program controls and may persist its
own internal arrangement, with explicitly delegated editing and explicit live
application. Code-defined/custom attenuation is equally valid. Service imports
are requests, never grants; secrets are supplied as operations, never credential
bytes in a child's configuration. A Warden is a provider role, not necessarily
one process or hardware backing.

The decision does not choose a universal matrix schema, mandate a program's
subtree as its realm, settle object naming/copy references, or implement Warden,
authentication or encryption. The initial owner still furnishes user matrices;
removing that special knowledge is the next implementation milestone. Existing
Rust import/launch contracts document their bounded evidence; no ABI or runtime
behavior changed in this documentation pass.

## Independent-copy decision (2026-10-18)

The owner accepted [independent copy semantics](wiki/spec/storage/realms_and_objects.md):
hierarchy and backing are distinct, CoW is the default sharing model, and edits
to either original or clone preserve the other. Start with immutable replacement
writes; exclusive-use reuse can follow later without changing semantics.
Retained views and readers count when deciding whether backing is reclaimable.

The Rust store already shares immutable content with independent edits, but
subtree copies still allocate numbered metadata per descendant. Shared hierarchy
roots are not implemented. Logical IDs, copied configuration references and
aliases remain the narrowed owner question; view-relative IDs are a proposal,
not an accepted answer. This pass changes documentation and protocol comments
only, with no runtime or wire changes. Markdown links and diff checks were
verified; runtime tests were not rerun for this documentation change.

## Persisted path-reference decision (2026-10-18)

The owner accepted [paths within the supplied view](wiki/spec/storage/realms_and_objects.md)
for ordinary persisted references. Copies preserve path strings; missing targets
fail lookup, with no source-realm fallback or dependency repair. Rename/move may
break a path. Stable persisted IDs and aliases are deferred, so the copied-reference
owner question is closed rather than retained as a blocker.

The guest still uses numbered objects and has no named path lookup. Name/kind
representation is the remaining owner decision for matrix startup paths; internal
number allocation is not itself a public stable-reference contract. No runtime or
wire behavior changed in this documentation/comment pass. Local Markdown links
and diff checks passed; runtime tests were not rerun.

## Hierarchy name and media-type decision (2026-10-18)

The owner confirmed explicit [hierarchy metadata](wiki/spec/storage/realms_and_objects.md)
for names and media types, enabling indexed database filtering without filename
searches or opening every body. Extensions remain program conventions; importers
and explorers may infer and set type metadata when authorized. A declared type
is not proof of valid contents or permission to execute them. Content and
children may coexist, and the same backing can have different metadata through
independent entries.

The names/kinds owner question is closed. The next storage work is implementation:
metadata records, named lookup and scoped type filtering, followed by matrix-owned
startup paths. The guest still has numbered objects without these features.
No wire or runtime behavior changed in this documentation/comment pass. Local
Markdown links and diff checks passed; runtime tests were not rerun.

## Named realms and matrix-owned startup implemented (2026-10-18)

This completes the first implementation of the preceding decisions. F5 still
boots the ordinary desktop. The outer owner resolves `Login` and `Users`, grants
the selected user its ceiling, collects the picker, then requests `CONFIGURE`.
The user matrix loads its own `Matrix` object, resolves the saved launcher path,
and installs launcher/app placement and budgets. It no longer relies on the
outer owner knowing internal object numbers or observing its current app.

Follow [hierarchy.rs](source-rs/platform/libraries/database-objects/hierarchy.rs)
for edge metadata, exact name and type indexes, and atomic revision-checked
updates; [provider dispatch](source-rs/platform/services/storage/hierarchy.rs)
and [Scope](source-rs/platform/libraries/user-runtime/storage.rs) connect those
operations across IPC. Names/types are bounded to 32 UTF-8 bytes each; relative
lookup is bounded to 256 bytes, permits `.`, and rejects parent traversal and
absolute paths. Named siblings are unique with exact comparison. FILTER returns
up to six direct children of an exact media type. These lab bounds do not freeze
the eventual schema or type vocabulary. Pins still preserve content revisions,
not complete historical hierarchy metadata.

The chooser uses the executable-type index and hierarchy names. Its hidden
template is resolved by path; its hidden launcher comes from saved configuration.
NEW copies the template as `User N`, preserving descendant names/types and path
strings while sharing content. A misleading extension does not produce an app
tile. Unknown legacy metadata remains representable; there is no explorer or
metadata-editing UI yet. Name collisions refuse rather than overwrite.

[Configuration](source-rs/platform/contracts/matrix_config.rs) is the stock
matrix's bounded 104-byte `CTHMAT01` record: launcher path, page/reader budgets,
margin, launcher width, gap, app surfaces and key mask. The
[loader](source-rs/platform/services/matrix/service/children/configuration.rs)
requires empty child slots, checks geometry/key limits and uses ordinary
admission. Failed startup returns to the picker. No settings editor or live
apply is implemented. Parent grants remain authoritative; saving policy never
expands them. Current initial defaults are part of the explicit host migration;
move their selection into distribution composition before supporting alternate
stock arrangements or a settings workflow.

Each matrix remembers the source and scope of its own live child grants.
`RESTORE_STORAGE` restores those after provider replacement; successful explicit
revocation and child death clear remembered policy. Matching live grants are
left alone. Failed local storage exchanges retire their stream so a late reply
cannot be consumed as the next request's reply.

[Stock migration](source-rs/distribution/migrations/README.md) first refreshes
exactly recognized program copies, including old user matrices, then adds names,
indexes and configuration without renumbering or rewriting saved application
data. Existing metadata/configurations survive repeats. The exact old matrix
manifests added to the allowlist were checked against retained successful boot
fixtures; arbitrary matrix-shaped bytes are not accepted. All guest migration
tests used image copies; the live interactive disk was left untouched and F5
will run the normal upgrade on its next launch.

Evidence for this change (paths relative to `build/boot-harness-rs/`):

- `cargo test --locked` passed, along with 12 database-object host tests and
  30 storage-lab engine tests. Contract tests include invalid configuration
  geometry/key masks. Database tests cover stale/duplicate metadata updates,
  copied indexes, deletion, quota and reopening; migration tests preserve other
  roots and saved state and verify idempotence.
- Userspace and UEFI clippy, desktop shell/notes-probes/trace-storage and provider
  recovery/trace feature checks, host database/storage-lab clippy with warnings
  denied, source-layout checks (35 Omega/87 Rust edges), five layout tests,
  formatting, Python compilation and diff checks passed. No new ARM64 boot or
  full kernel smoke claim is made for this userspace/storage change.
- `desktop/20261018T211244773335Z`: default F5 creation, Notes edit/duplicate,
  app close/switch, cold boot and exact host oracle (241 records) passed.
- `hierarchy/20261018T211015023781Z`: updated launcher path after rename,
  extension-looking text exclusion, migration preserving owner edits, and
  missing launcher returning visibly to the picker passed.
- `objects/20261018T210147888941Z`: guest metadata/lookup/filter scope refusals,
  valid in-scope query, existing object operations and both cold-boot oracles passed.
- `colors/20261018T210311121562Z`: copy of the retained two-user interactive
  disk upgraded, launched/saved/reopened Colors and cold-booted successfully.
- `colors/20261018T211700524443Z`: final recovery code on a fresh one-user
  fixture survived the injected lost committed-save reply, provider replacement,
  explicit child grant restoration, reconciliation and cold boot.
- `matrix-storage-test`: final recursive scope/admission/replacement suite passed,
  including restoration preserving revocation, the outside child's source, and
  no inheritance of a dead child's remembered grant.

One limit found: recovery-profile installation on the older retained two-user
disk copy exhausted the 4 MiB engine extent before QEMU started
(`colors/20261018T210353517960Z`, requested 4,214,784 bytes, capacity 4,186,112).
Normal migration/boot on that retained disk copy passed; recovery passed on the
fresh fixture. This accumulated-upgrade capacity issue is not fixed by raising
logical quota and remains work for the installer/storage lifecycle. Never
silently replace or reformat a user's image to hide it. The prior intermittent
ATA timeout and quiet fault-gate occurrence remain recorded above, unresolved.

## Saved layout editor and reliable profile switching (2026-10-18)

The normal user launcher now has SETTINGS, also reachable with F2 when it has
focus. It edits strip width (192ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã…â€œ320 pixels) and outer spacing (8ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã…â€œ40 pixels).
Save writes that user's `Matrix` configuration; Cancel discards local edits and
returns to app tiles. The current app keeps running in the same position.
Restart QEMU and select the user to apply the saved layout. This is a bounded
saved-layout editor, not live reconfiguration or a budget/launcher editor.

Start at the chooser's [session](source-rs/distribution/applications/chooser/session/session.rs):
route local input to [selection](source-rs/distribution/applications/chooser/session/selection.rs)
or [settings](source-rs/distribution/applications/chooser/session/settings/settings.rs),
then publish the selected view. Settings owns its draft, storage decisions and
small panel renderer. The launcher already has an explicit grant over the user
realm; apps still receive their own subtrees. No new kernel or platform authority
was added. Its zero reader-page allowance exposed an early bulk-read refusal;
the final code reads this 104-byte record through inline replies checked against
one revision, and retains the existing 48-page rendering allowance.

Saving checks the path still names the same object and refuses an intervening
different revision/content. Publication uses the expected revision. A lost
acknowledgement remains uncertain; clicking Save again reads back the desired
content before deciding whether a write is needed. Unsaved/uncertain work reports
LOSS. Cancel does not undo a write that already committed. A provider outage can
make opening the panel say CANNOT OPEN; retry after recovery. There is no editor
for resolving concurrent conflicting drafts yet.

Initial values moved from host-tool constants to
[distribution/matrix.conf](source-rs/distribution/matrix.conf). The host encodes
those defaults, preserving existing template/user configurations during upgrades.
Changing the file does not silently reset saved arrangements.

Testing revealed a separate build issue: after a recovery profile, a normal
desktop run reused a kernel bundle containing the quiet fault gate. The failed
default proof is `desktop/20261018T213314127325Z`; its log shows gate arming and
stalling during Notes save. Cargo's package output paths were reused across
feature sets, and cached outputs could restore older timestamps. The harness
now [captures each built role](tools/boot-harness-rs/artifacts.py) at a path keyed
by content digest, immediately after building it. The kernel build already
watches those environment paths, so changing feature bytes invalidates the
bundle independently of timestamps. Store-program trimming also uses each
captured role, avoiding later same-package builds replacing its input. Keep
different profile builds sequential; this is not a concurrent-build coordinator.

Verification (artifacts under `build/boot-harness-rs/`):

- `desktop/20261018T213552026966Z`: final normal desktop after recovery-profile
  use passed user creation, Notes editing/duplication, close/switch, cold boot
  and exact retained-record oracle (247 records).
- `settings/20261018T213712506523Z`: final normal three-boot proof passed Cancel,
  Save, unchanged live app placement, persisted dimensions after restart, and
  a newly created second user retaining independent template defaults. Installer
  reruns left the edited image byte-identical.
- `settings/20261018T213417759204Z`: two-boot recovery proof passed a deliberately
  lost committed settings-save reply, provider replacement, reconciliation and
  cold layout restoration. Its trace contains exactly one publication despite
  the retry. The third-boot new-user check belongs to the normal proof; running
  it with the gate rearmed also stalls Colors' initial save and can transiently
  refuse opening Settings, which is a separate fault from the settings save.
- `cargo test --locked`, all 30 storage-lab engine tests, userspace clippy and
  storage-lab clippy with warnings denied passed. The artifact regression test
  checks distinct feature bytes with identical timestamps, preserved captured
  inputs, and returning to the earlier content identity. Layout checks (35/87
  package edges), five layout tests, formatting, Python compilation and diff
  checks passed. Kernel smoke and ARM64 boots were not rerun.

At this milestone the prior fixed-volume upgrade capacity limit remained. These
GUI/fault proofs used disposable copies; the interactive disk was left untouched.
The follow-up below repairs the actual F5 installation path.

## F5 upgrade capacity repair (2026-10-18)

The owner's normal F5 launch failed before QEMU started: seeding updated apps
requested 4,214,784 bytes from the 4,186,112-byte engine capacity. Reproduced on
a copy of the actual two-user disk. Physical compaction alone allowed the
same installation to finish, without collecting history or enlarging the volume.

[database_image.py](tools/boot-harness-rs/database_image.py) now invokes offline
[compaction](tools/storage-lab/engines/volume/compact.rs) before updating existing
media whose installation receipt changed. Unchanged launches never open the
database; newly formatted media skips maintenance. The host locks the image,
saves and syncs a full `.before-compact-<BLAKE3>.raw` backup, compacts, cold-opens
and checks all table records and volume identity. A conflicting backup is
refused. Maintenance failure stops the install and does not write its receipt.
No kernel, guest storage format, quota, hierarchy or saved-data semantics changed.

Verification:

- All 31 storage-lab tests and clippy with warnings denied passed. The new
  integration test checks exact records across cold reopen/repeated maintenance,
  retained history, reclaimed engine length, exact backup bytes, backup-conflict
  refusal and unchanged image size/outside-region guards.
- Five Python installer/artifact tests passed, including maintenance-before-seed,
  unchanged receipt bypass and refusal without seeding or recording success.
- `colors/20261018T220311689389Z`: a copy of the actual retained two-user disk
  compacted from 2,109,440 to 1,425,408 logical engine bytes, preserving all 394
  records. App installation/migration, login, Colors save/close/reopen and cold
  boot passed. First and cold app launches took about 0.45 and 0.39 seconds.
- The exact F5 command then upgraded the actual interactive disk successfully.
  Its pre-maintenance backup is beside the disk, digest
  `552ed73fd21d7be7b1abc1fe1b4e5a8e0279fba6870058006294befd26f57201`.
  QEMU displayed both existing user tiles; the actual window capture is
  `f5-diagnosis/actual-window.png`. The GUI was left open for the owner.
- Layout and diff checks passed. Guest code was unchanged; kernel/ARM64 and the
  full fault suite were not rerun.

This fixes the observed reclaimable-space failure, not every capacity limit.
The extent is still 4 MiB; truly full media or compaction without scratch space
can still refuse. Volume growth and installer-wide atomicity remain future work.
Backups accumulate only on changed installations and are never silently deleted.
The prior ATA timeout and quiet fault-gate caveats remain unresolved.

## Authorized live layout and rendering invalidation (2026-10-18)

SETTINGS / SAVE now persists the user's arrangement and requests immediate
geometry application. Colors stays running, Notes keeps dirty drafts and close
protection, and later launches use the updated app rectangle. Cancel still
abandons only local edits. Saving and applying are separate operations: APPLIED
means both succeeded; SAVED; RESTART means the durable save succeeded but live
application was refused. Save can be retried without repeating an acknowledged
write. Budgets, launcher selection and key-mask changes still require startup;
editing the configuration object alone does not automatically change live policy.

The stock matrix explicitly grants its configured launcher's interaction endpoint
layout authority, bound to its incarnation and checked against its current view
epoch. Ordinary apps, the picker, storage endpoints and stale incarnations cannot
use it. Authority is checked before reporting busy owner operations. The new
`APPLY_LAYOUT` request accepts only margin/strip width/gap, validates both
rectangles before changing either, preserves current key masks and allowances,
and does not revive a revoked sibling view. Controller revocation/death or
parent-binding replacement clears the grant. Equal geometry is idempotent.
Start at [configuration.rs](source-rs/platform/services/matrix/service/children/configuration.rs)
for the grant and standing-placement update, then
[layout.rs](source-rs/platform/services/matrix/world/layout.rs) for authority and
geometry. The protocol is owned by platform/contracts/matrix.rs; no kernel
change or owner-control endpoint was exposed to children.

Actual resizes can invalidate a draw halfway through a frame. Chooser, Notes and
Colors now propagate rendering refusals to their session, verify the view really
changed or was revoked, and rebuild surfaces while keeping application state.
Unrelated refusals under the same live epoch remain errors. Colors' ordinary
surface setup no longer executes its adversarial lab probes; those remain in the
lab build. Existing focus/held-key suppression handles changed extents; app-local
button tracking resets for a new epoch.

Verification (artifacts under build/boot-harness-rs):

- Workspace host tests passed. The new policy test covers absent/sibling/stale
  authority, malformed/overflow/no-room requests without partial placement,
  stale drawing epochs, idempotence, held-key suppression, preserved revocation
  and controller teardown. Userspace checks and clippy with warnings denied
  passed for matrix, chooser, Notes including render probes, and desktop shell
  and lab variants. Layout checks, five layout tests and formatting passed.
- `settings/20261018T221332506921Z`: normal three-boot proof passed Cancel,
  immediate Colors layout, repeated resizing with a dirty Notes draft, continued
  editing, app-owned close protection, save/close, later app placement, persisted
  geometry and independent new-user defaults. The earlier run's exact bitmap
  comparison failed because the pointer covered the note; the corrected test
  moves the pointer outside the compared bitmap before checking.
- `settings/20261018T221519348512Z`: test-only Notes gates forced a resize after
  the first surface allocation and another during bitmap rendering. Both stale
  frame paths executed, Notes loaded its document exactly once and typing worked
  afterward. These gates compile only with `render-probes`, never normal F5.
- `settings/20261018T221620237698Z`: lost committed-save reply, provider recovery,
  explicit storage restoration, readback reconciliation, live application and
  cold boot passed. The trace still shows exactly one settings publication.
- The matrix desktop suite passed nested clipping, focus, subtree replacement
  and cold colors using the lab build's surface probes.
- `desktop/20261018T221823899504Z`: the final default F5 composition passed
  user creation, app selection, Notes editing/duplication, close choices,
  repeated replacement and cold boot. The independent host oracle verified
  all 243 retained records, including exact hierarchy/history/quota and the
  empty ungranted root. This final build restored the normal F5 artifacts after
  the test-only profiles.

The interactive QEMU session and saved disk were left alone throughout this
milestone. Tests used disposable copies. F5 installs updated recognized programs
on the next launch; close the old QEMU window before launching another instance
against the same disk. No kernel or ARM64 behavior changed; those boot suites
were not rerun. Live child replacement, resource-budget reconfiguration and
startup/crash UI recovery remain separate next work. No new owner decision was
needed for this bounded geometry request.


## F5 retained-history quota repair (2026-10-18)

A subsequent F5 failure was reproduced from a copy of the actual two-user disk:
the Rust build passed, compaction preserved 614 records and reclaimed physical
space, seeding updated five stock objects, but `upgrade-app-close` aborted with
`label upgrade exceeds root quota`. Retained record bytes and physical engine
space are separate limits; physical compaction alone cannot clear this failure.

Stock refresh now plans recognized copies before removing any template history,
stages all replacement heads, and, only under quota pressure, runs the same
head/pin reachability collector as the guest inside its transaction. It commits
the replacements and collection together. Current files, customized programs,
hierarchy, identifiers and pinned revisions survive; unreferenced history may be
collected under the existing retention semantics. Live/pinned content that still
cannot fit refuses and rolls back the entire refresh. Pre-upgrade backups remain.
The 1 MiB root quota and 4 MiB interactive extent are unchanged. Seeding and
hierarchy migration remain separate transactions; installer-wide atomicity and
true live-capacity exhaustion remain limitations.

The reusable transaction pass is `collect_records` in
[collection.rs](source-rs/platform/libraries/database-objects/collection.rs).
Guest `Store::collect` still commits one bounded pass with identical policy.
The host [stock refresh](tools/storage-lab/engines/volume/labels.rs) calls it
only after its replacement plan has been staged, preserving stock recognition
from history. The host tool now depends on this shared library instead of
implementing another collector; redb's unified experimental API needs explicit
ReadableTable imports in the existing adapters.

Verification:

- All 32 storage-lab tests and its ordinary clippy check passed. New pressure
  coverage checks cold records, current data/custom code, durable pins and their
  chunks, exact quota, backups, image guards, repeat idempotence, and complete
  logical rollback when live content exceeds quota. The synthetic quota fixture
  has 8 MiB physical space to isolate the 1 MiB logical admission limit.
- Workspace host tests and all 12 database-object host tests passed, including
  interrupted collection and fixed-extent reuse. Database-object clippy, source
  layout, formatting and diff checks passed. A broader storage-lab `--all-targets`
  clippy run found the existing `hierarchy.rs` test module before `inspect`; the
  ordinary clippy target passes. No kernel/ARM64 checks were rerun.
- `colors/20261018T223112972028Z` upgraded a copy of the actual retained disk:
  charged bytes fell from 1,310,925 to 1,041,964 while refreshing eight stock
  copies. Login, Colors launch/save/close/reopen, and cold persistence passed.
  Picker startup took 3.80 s and 10.95 s; app launches about 0.48 s and 0.59 s.
- The old interactive QEMU remains running against `cathedral-storage.raw`;
  the real image was not upgraded underneath it. Close that window before F5;
  the next launch applies this repair automatically without a reset.

At the time of this repair, launch-outcome/startup-failure work remained
uncommitted. The following milestone completes that work and its verification.


## Recoverable child launch and scoped outcomes (2026-10-18)

The ordinary launcher now displays CANNOT OPEN on refused admission and APP
STOPPED when its admitted program faults or returns nonzero. It remains available
for another program; a clean exit is not labeled a crash. User CONFIGURE failure
returns to a fresh picker with CANNOT OPEN. Startup cleanup accepts a child that
has already been collected, fixing the race where failed startup panicked the
outer desktop. Root Login/Users lookup and initial provider admission still assert.

A successful child LAUNCH returns an observation receipt. LAUNCH_STATUS reports
only that requesting endpoint's latest successful launch, tied to its incarnation
and the exact kernel task handle. Counters are per observer. No PID, fault details,
other caller's launch count or later occupant of a reused slot is disclosed.
Requester teardown clears observation authority. Terminal task collection wakes
the observer. Start at [children.rs](source-rs/platform/services/matrix/service/children/children.rs)
for admission/placement, [lifecycle.rs](source-rs/platform/services/matrix/service/children/lifecycle.rs)
for outcome/cleanup, and [launches.rs](source-rs/platform/services/matrix/world/launches.rs)
for the bounded receipt policy. Rust mod.rs remains plumbing.

Surface-allocation or placement refusal now stops and refunds the just-admitted
child instead of reporting an invisible success. Deliberately headless policy
remains valid. A failed LOSS send returns a refusal rather than killing the matrix.
More generally, a broken control direction is not proof of process death: owner
probes and close handling check a nonblocking kernel task wait and preserve a live
process with BUSY. Terminal outcomes are retained until cleanup so they are consumed
only once. Explicit owner STOP remains distinct authority. Pending close can lose
a race to owner STOP, and a task can exit during a recursive loss survey without
leaking its slot; deferred disconnects remain remembered until cleanup.

Verification:

- Workspace host tests passed earlier in this milestone; all 23 matrix policy
  tests passed after review, including receipt isolation, endpoint reuse, exact
  task matching and counter exhaustion. Userspace clippy and matrix startup-fault,
  Notes startup-fault/admitted-fault, desktop shell and storage-lab variants passed
  with warnings denied. All 32 storage-lab tests and ordinary clippy passed.
  Source layout, Rust formatting, Python compilation and diff checks passed.
- `lifecycle/20261018T223552633573Z` and
  `lifecycle/20261018T224234066969Z` passed four copied-image scenarios: malformed
  program, Notes death racing admission, Notes fault after a completed
  interaction round trip, and user-matrix death inside CONFIGURE. Each fails three
  times, then launches/saves/closes a healthy Colors. The user-fault case creates
  and enters a healthy second user from the unchanged template. Normal exit is
  checked against a delayed false crash notice. Source fixture bytes stay intact.
- Earlier lifecycle failures were checker assumptions: panic location and message
  span separate lines; the avatar color was wrong; and an early task fault can
  legitimately return admission refusal if it wins storage binding. Separate early
  and post-admission fault gates now verify both behaviors. Fault-only features
  are absent from ordinary F5; the copied fixture alone receives injected programs.
- `--matrix-storage-test --release` passed repeated placement rollback, denied
  foreign/stale receipts, monotonic own receipts, reuse by an owner-launched task,
  closed control-request and control-reply directions with a surviving task,
  uncertain owner health probes, owner STOP during pending CLOSE, and exit during
  LOSS followed by successful replacement. Existing storage scope/recovery checks
  also pass. `--matrix-test --release` passed recursive attenuation, dirty/delayed
  loss protection, clean subtree replacement and recovery.
- `desktop/20261018T224335105991Z` passed the normal F5 composition: user creation,
  Notes editing/duplication, save/discard/cancel, app selection and cold persistence.
  The independent host oracle verified all 247 records, exact hierarchy/history/
  quota and the empty other root. No interactive disk or running QEMU was changed.
- `colors/20261018T224602610580Z` rechecked the final build on a copy of the
  retained two-user interactive disk. Upgrade/collection, login, Colors save,
  close/reopen and cold boot passed; both app launches took about 0.58 s. This
  keeps the F5 quota repair covered as the stock executables change.

No kernel ABI or authority boundary changed, and no owner decision was required.
Kernel/ARM64 suites were not rerun. The broader lifecycle queue remains open:
restart a failed configured launcher within its own matrix, recover root startup
failures, and offer explicit recovery for an unresponsive app without silently
throwing away work. Previous storage capacity, ATA timeout and quiet fault-gate
limitations remain as recorded above.

## F5 already-running diagnosis (2026-10-18)

Another reported F5 failure had an existing interactive QEMU (PID 66056 at
diagnosis) still holding `cathedral-storage.raw`. A read-only exclusive Windows
open returned sharing violation 32. The default harness now checks availability
before compilation or replacing boot files, and tells the user to save/close
the previous QEMU and retry. It leaves that guest running. The probe is not a
reservation; installer/QEMU locks remain authoritative. Build-only and disposable
tests remain available while the interactive disk is busy.

Six database-image harness tests passed, including a real Windows sharing
conflict that refuses before any build and succeeds after the handle closes.
The exact F5 command now exits immediately with the actionable diagnostic.
Python compilation, layout and diff checks passed. No Rust code changed.
`f5-diagnosis/20261018T225227024191Z` reused the last verified default build on
a disposable copy of the current two-user disk: upgrade, picker, login, Colors
save/close/reopen and cold boot passed. Picker readiness was 3.62 s and 9.33 s.
The source disk hash was unchanged. The existing interactive QEMU and its boot
files were untouched; closing it is still necessary before the next F5 launch.

## Local configured-launcher recovery (2026-10-18)

Successful CONFIGURE now arms one local retry of its launcher after a fault or
nonzero return. The child task's actual outcome decides eligibility; a clean
return, explicit STOP or owner policy change does not restart it. Collection
releases the old launcher before replacement. The same asynchronous HEAD/BULK
path used by normal launches reloads its executable without blocking interaction
on the separate parent stream or retaining an extra code reader indefinitely.

Start at [recovery.rs](source-rs/platform/services/matrix/service/children/recovery/recovery.rs).
It snapshots current placement, page policy and storage identity, waits for old
deferred events/control replies/publication to retire, then admits fresh endpoints
and issues fresh storage/layout grants. The app's task, scene, geometry epoch,
focus and document remain intact. No launch receipt transfers to the replacement.
It does not reload Matrix configuration or reinterpret an object number in a new
realm. Parent display generation, storage connection/realm incarnation, explicit
owner changes and new slot occupants prevent stale policy from reviving grants.
A loss survey also reports uncertainty if recovery becomes pending during it.

The retry is consumed on failure too. There is no timed retry reset, manual
launcher restart UI or recovery of the dead launcher's own unsaved Settings
draft. Reboot/configure can start a new session; live apps can still save and
close after retry exhaustion. Initial root Login/Users lookup and provider
startup remain the next lifecycle frontier, followed by explicit operator
recovery for unresponsive apps and an exhausted launcher.

Verification:

- Workspace host tests, all 25 matrix policy tests, all 32 storage-lab tests and
  ordinary storage clippy passed. New policy tests preserve app scene/epoch/focus/
  queued input while replacing the launcher and reject ancestor revoke/regrant
  or capacity changes. Userspace clippy and trace/startup/fault/owner-probe feature
  sets passed with warnings denied. Layout, formatting, Python and diff checks pass.
- `launcher-recovery/20261018T230048782616Z` passed five copied-image scenarios:
  dirty Notes survives launcher fault/replacement, replacement changes live layout,
  second fault stays stopped, Notes still accepts input/save/app-owned close,
  and its exact rendered text survives cold boot. Separate cases verify clean
  launcher exit, two immediate startup faults bounded to one retry, real routed
  owner STOP, and owner storage revocation before a fault. Notes loads once per
  boot; source fixture bytes are unchanged. An earlier three-case run also passed.
  The chooser fault keys and desktop owner probes are absent from ordinary F5.
- `desktop/20261018T230320916031Z` passed normal user creation, app selection,
  Notes duplication, save/discard/cancel, replacement and cold persistence. The
  independent host oracle verified 249 records and exact hierarchy/history/quota.
- `--matrix-storage-test --release` and `--matrix-test --release` passed after
  the loss-survey review, covering ordinary async launch, receipt scopes,
  control-channel failures, cleanup, recursive loss protection and delegation.
- `colors/20261018T230536053670Z` passed upgrade/login/Colors save/reopen/cold
  boot on a copy of the actual two-user disk. Upgrade collected unreachable
  history and charged 1,046,168 of 1,048,576 allowed bytes. Collection stops once
  the transaction fits, so this charge does not establish the live-content floor:
  it can still include unreachable history. Launches took about 0.58/0.59 s.
  Growing live content may reach genuine capacity; collection is not unlimited.
- Desktop, Colors and the new recovery fixture now build identical compositions
  into separate ESPs, avoiding replacement of an active F5 guest's boot files.
  The interactive disk and old QEMU were left untouched. No kernel ABI changed;
  kernel boot/ARM64 suites were not rerun for this userspace milestone.

## Independent Colors application (2026-10-18)

The installed Colors program now starts at
[applications/colors/main.rs](source-rs/distribution/applications/colors/main.rs),
then its named session entrance. Previously it was another role of the desktop
startup executable, so every owner change also enlarged the stored app. Its
trimmed release image is now 45,288 bytes versus 78,104 before extraction.
The profile installs it at the same template position, and normal stock refresh
preserves existing user object identities, metadata and saved colors.

The recursive desktop lab links the same Colors library, selecting its probe
loop at compile time and supplying the lab-owned command handler. Ordinary
Colors selects the app loop with its close button; it contains no provider
startup. Shared state/surface behavior remains in the application, with probe
composition and assertions under the desktop lab. No kernel ABI changed.

Verification:

- Workspace host tests, userspace clippy, desktop database-probe and shell trace
  feature checks passed. Layout (35 Omega/92 Rust package edges), formatting and
  diff checks passed.
- `colors/20261018T231229384277Z` upgraded a copy of the retained two-user disk,
  selected user 2, saved/reopened Colors and verified its color after cold boot.
  Launches took 0.407/0.438 s. The source interactive disk was untouched.
- `--matrix-desktop-test --release --timeout 180` passed the shared app's nested
  rendering, physical input/focus, quotas, storage/provider/subtree recovery and
  persisted colors after a fresh QEMU process. The repeat captured an explicit
  successful harness exit after the earlier process handle was unavailable.

Next: root startup still asserts on missing Login/Users or rejected picker
admission. Give that failure a visible, manually retryable distribution screen
that does not depend on loading another program from the broken store. Provider
startup failure and recovery controls for unresponsive apps remain separate
unfinished work. No ARM64 boot was rerun for this userspace extraction.

## Visible root login recovery (2026-10-18)

F5 now shows **SIGN-IN UNAVAILABLE / TRY AGAIN** if root Login/Users lookup
fails, the stored picker cannot be admitted, or the picker dies. Start at
[session/session.rs](source-rs/distribution/desktop/session/session.rs), then
[shell/login.rs](source-rs/distribution/desktop/session/shell/login.rs) for the
Picker/Recovery/Stopped phases. The separate
[recovery executable](source-rs/distribution/desktop/recovery/main.rs) is bundled
in the boot catalogue, so showing a failure does not require another successful
program read from storage. Ordinary picker and app code still come from storage.

The recovery child has local matrix drawing/input, no bound storage realm,
no devices and no descendants. The catalogue path currently supplies an unbound
storage endpoint which it ignores. It coalesces a button click or focused Enter
into one request on its owner-only control link; only the startup owner retries.
The owner keeps the screen through path reads and retires it before admission.
Repeated failure returns to the screen, with no automatic retry loop, disk
formatting or names/program repair. A dead recovery child does not itself trigger
admission retries. The path and program-read client deadlines now outlast the
matrix's storage verdict, rather than panicking at the old three-second limit.

The selected user continues to own the whole desktop, and the login child is
stopped before user CONFIGURE. Storage replacement restores a picker realm only
in the Picker phase; it never grants storage authority to the recovery screen.
This does not handle failure of initial display, input, root or storage provider
startup, which still precedes this phase and is the next lifecycle frontier.
Unresponsive apps and exhausted configured-launcher recovery still lack manual
operator controls. No owner semantic/trust decision was needed for this change.

Verification:

- `startup/20261018T232825283690Z` passed missing Login, missing Users, invalid
  Login ELF, picker startup panic and a one-time injected admission refusal.
  Each persistent failure accepts three mouse/keyboard retries and stays alive;
  the transient case returns to the real picker, enters a user and saves,
  closes/reopens Colors. Fault features are absent from the default composition.
- Each failed guest is cold-compared against its prepared snapshot: all tables,
  records, volume identity and guards are unchanged. Engine bookkeeping changed
  55 physical bytes. The new preservation oracle also rejected deliberately
  changed hierarchy records and guard bytes in negative controls. Source fixture
  bytes remain unchanged. Early checker failures were a miscounted title width,
  cursor occlusion and assuming the whole engine image stayed byte-identical;
  none required relaxing the record-preservation check.
- `desktop/20261018T232935589889Z` passed the normal F5 flow, app-owned close,
  save/discard/cancel and cold persistence. Its independent host oracle verified
  all 233 records and the exact hierarchy/history/quota. An earlier run passed too.
- `colors/20261018T233104082301Z` passed a deliberately lost save reply, provider
  replacement, explicit realm restoration, save reconciliation, reopen and cold
  persistence. The live user was retained and the retired picker was not revived.
- Workspace host tests, all 32 storage-lab tests, userspace clippy, shell/probe/
  trace feature checks and host-engine clippy passed. Layout (35 Omega/96 Rust
  edges), source formatting, Python compilation and diff checks passed. The two
  changed host helper leaves were formatted; the host lab's broader formatting
  check reports pre-existing drift in unrelated include files.

Interactive QEMU and its disk/ESP were untouched; all GUI checks used disposable
copies and separate ESPs. No kernel ABI changed. ARM64 boot was not rerun.

## Interactive storage-provider startup (2026-10-18)

The default owner now admits display, the root matrix and physical input before
opening storage. The boot-bundled screen shows **STARTING...** while
[persistence/startup.rs](source-rs/distribution/desktop/session/persistence/startup.rs)
polls the private readiness reply. The exact bytes must be `ready`; a matching
length is insufficient. Process death, a malformed reply or the 3,000-tick
readiness timeout collects that attempt and enables **TRY AGAIN**. Each click
authorizes one attempt; initial failure does not start an automatic retry loop.
There is still no formatting or repair fallback.

[provider.rs](source-rs/distribution/desktop/session/persistence/provider.rs)
owns one task, its control endpoints and pending readiness deadline. Both startup
and runtime replacement use it, so partial connection/readiness failure closes
the old stream and collects the old task before reusing its launch grant. The
session explicitly has no provider while offline. Lab setup/runtime replacement
retain their bounded synchronous readiness wait; the normal startup path polls.
Successful manual retry does not replenish the three automatic replacements
allowed later in the session. Live users are not eligible for startup recovery.

The recovery program's distribution-private STATUS command changes its card.
The owner revokes queued input and restores a fresh lease around that transition;
the program preserves observed button state, so holding over Retry through a
timeout does not request another attempt, and a later fresh click needs one press.
The recovery child retains no storage realm. A 64-page allowance accommodates its
32-page image and replacement; it is collected before the picker is admitted.

Verification:

- `provider-startup/20261018T234651309751Z` passed all five final cases: crash
  before opening, malformed five-byte readiness, a live pre-open stall through
  the normal timeout, repeated crashes, and two invalid volume headers. The stall
  kept physical pointer rendering responsive while a button stayed held over
  Retry; release/fresh click recovered into the real picker and a usable Colors
  that saved, closed and reopened. Repeated failures each required a fresh click
  with the cursor left in place. Crash/corrupt-header attempts left the complete
  disk byte-identical, and the source fixture stayed untouched. Fault features
  are absent from ordinary F5. Earlier checker failures had incorrect timeout
  and error-text expectations; the guest had correctly reached recovery.
- `startup/20261018T234529797906Z` passed all existing missing-path, malformed
  executable, picker-crash and successful manual login-retry cases. Its host
  oracle confirmed every record, volume identity and guard unchanged for failed
  attempts. This preceded the final button-state refinement, which the provider
  suite specifically exercises.
- `desktop/20261018T233828976820Z` passed the normal F5 flow and cold persistence
  after the startup reordering, with all 233 records checked by the host oracle.
- `colors/20261018T235149916300Z` passed a deliberately lost save reply, provider
  cancellation/collection, replacement and explicit grant restoration. The live
  user reconciled its save, reopened the app and retained its color across cold
  boot. This exercises the shared provider lifecycle after the startup refactor.
- `run.py --matrix-desktop-test --release --timeout 180` passed the recursive
  desktop's storage stalls, provider replacement, tree recovery, input confinement
  and cold persistence. The earlier completed run had no recoverable process
  result, so this final run supplied an explicit successful exit and both PASS
  reports. Artifacts are under `matrix-desktop-test/`.
- Workspace host tests, userspace and affected shell/lab/provider clippy feature
  checks passed. The two restart-budget tests run through the new
  `recovery_policy_test.py`, which supplies actual contract constants and an
  isolated host unwind build; the previous documented standalone command was
  stale and lacked the contract dependency. Layout remains 35 Omega/96 Rust edges.

Initial display/input/root admission failures remain uncovered, as do explicit
controls for a live unresponsive app or exhausted launcher recovery. No kernel
ABI or driver authority changed; no owner-level semantic decision was required.
The active interactive QEMU and its disk/ESP remain untouched. ARM64 boot was
not rerun for this distribution lifecycle change.


## Bounded early bootstrap recovery (2026-10-18)

[Admission](source-rs/distribution/desktop/session/admission/admission.rs) now has
an explicit partial-attempt lifetime and named display, root/funding and input
steps. Its F5 startup boundary permits two attempts total before storage or any stored user is admitted. A failed
stage collects input/root/display in that order, closes endpoints still held by
the owner and restores root funding. Successful endpoint moves are removed from
the partial-attempt inventory immediately. Input readiness now has a 300-tick
bound and checks exactly `ready`; root readiness must decode to an all-zero HEALTH
reply, and display INFO must decode successfully with supported nonzero extents.

Every rollback compares the owner's available process slots, physical reservation
and charge, page/channel limits, private/reader charges and dynamic-handle charge
with its pre-attempt snapshot. Root page/channel allowances remain in that idle
account: combining the full allowance with the owner's would exceed its individual
cap. Root process slots and physical reservations are reclaimed, then explicitly
re-funded for the next attempt. These checks use existing syscalls; no kernel ABI
or device authority changed.

Exhaustion logs the stage (display 0, input 1, root 2) and error. For root/input
failures, the owner re-admits display alone and renders **COULD NOT START / PLEASE
RESTART**, then blocks. This requires no matrix, input, storage or raw framebuffer
access by the owner. Display failure itself cannot provide graphical recovery;
the terminal reports it and the initial program returns. No user session is torn
down by this path, and no storage is opened, formatted or repaired.

The fault suite injects pre-initialization task exit, a blocked readiness reply,
malformed readiness, and admission refusal after staging but before endpoint
moves. Features are absent from the stock profile. These are not exhaustive
hardware-initialization or live-driver-replacement tests.

Verification:

- All 24 bootstrap cases passed across the recorded runs. Under `bootstrap/`,
  `20261019T000429090327Z` passed all eight display cases and transient input
  death, then caught the restart message exceeding the display protocol's
  16-character text limit. The corrected **PLEASE RESTART** screen passed repeated
  input/root death in `20261019T000742913873Z`; `20261019T000822525080Z` passed the
  remaining thirteen cases. Every transient failure reached a real user and a
  Colors app that saved, closed and reopened. Repeated failures stopped after
  two attempts, returned the measured budgets, and left the entire guest storage
  image byte-identical. Source fixtures were unchanged. Root/input terminal
  screens were checked from scanout; one was also inspected visually.
- `desktop/20261019T000008384217Z` passed the ordinary F5 user/app flow, app-owned
  close and save/discard/cancel, cold persistence and the independent oracle's
  exact 233-record check. This preceded the final diagnostic text correction and
  function relocation; the later fault cases exercised the final startup path.
- `run.py --matrix-desktop-test --release --timeout 180` passed with the final
  admission layout: recursive isolation, provider/tree recovery, physical focus
  and both saved colors after cold boot. This covers the catalogue labs sharing
  the new admission, input readiness and cleanup code.
- Workspace host tests, userspace clippy, shell/probe/trace and catalogue/database
  feature checks passed. Layout (35 Omega/96 Rust edges), source formatting,
  Python compilation and diff checks passed. No kernel ABI changed; ARM64 boot
  was not rerun for this distribution-level change.

Next: explicit operator controls for an unresponsive app or exhausted local
launcher recovery. Initial recovery is bounded now; live display/input/root loss
still needs a separate policy that accounts for unsaved user work. The active
interactive QEMU and its disk/ESP remain untouched; all tests use separate media.

## Explicit app termination, 2026-10-18

The F5 launcher now offers END APP above Settings while its delegated app slot
is occupied. F3 with launcher focus opens the same warning. Cancel is the default
keyboard choice; a fresh confirmation cancels the observed app/subtree without
requiring cooperation from a broken app. Ordinary app X and sibling CLOSE retain
their loss-protection rules. The existing three tile surfaces render the warning
and choices; the optional footer stays within the launcher's current page grant.

The matrix owner can grant/revoke management independently of launch receipts.
The ledger binds authority to the authenticated endpoint incarnation; queries
issue endpoint-local tickets for exact kernel child handles. There is no PID or
caller-supplied slot selector. Regrant, revocation and task retirement invalidate
tickets; counters reset only when the manager connection dies. CONFIGURE and its
bounded launcher restart explicitly grant the selected launcher management of
the app slot. An explicit owner policy change cancels automatic recovery, so a
restart cannot restore revoked authority. Teardown uses the existing task,
surface, endpoint and funding cleanup. No kernel ABI or stored schema changed.

Verification:

- `management/20261019T002724060034Z` passed original-launcher, recovered-launcher
  and owner-revocation scenarios. Physical input exercised both Cancel actions,
  forced termination of a non-yielding dirty Notes task, replacement Notes with
  only saved content, Colors save, and two cold reboots. Real IPC denied app
  self-grants, management on app/storage endpoints, retained tickets after target
  replacement, and retained tickets after owner revocation. Revocation also
  cancelled launcher auto-recovery while Notes kept its draft and could save/close.
  The full confirmation was checked in scanout and visually inspected. The source
  fixture was byte-identical afterward. Earlier two-mode evidence is under
  `management/20261019T002509885704Z`.
- `desktop/20261019T002848038329Z` passed the unchanged default F5 composition,
  user creation, app-owned save/discard/cancel, app replacement, cold persistence
  and the host oracle's exact 235-record check.
- `run.py --matrix-storage-test --release --timeout 180` passed existing recursive
  delegation, scoped launches/receipts, failed control links, owner-stop/close
  races, loss surveys and replacement. These exercise the shared cleanup path;
  the new force-stop-versus-pending-control interleavings are not exhaustively
  injected. Active storage/publication and loss survey gates remain bounded by
  existing deadlines; BUSY needs another explicit confirmation.
- Workspace host tests, userspace clippy and changed fixture-feature clippy passed.
  Layout (35 Omega/96 Rust edges), format, Python compilation and diff checks pass.
  No kernel or architecture changes; ARM64 was not rerun.

This is ordinary manager authority with a distribution confirmation, not proof
of human consent or an unspoofable system escape. The owner question about minimal
trusted platform chrome remains open; its outdated description of the Rust
status-app experiment has been corrected. Next: explicit recovery after launcher
retry exhaustion, then live provider-loss behavior without discarding app drafts.
The user's existing interactive QEMU, saved disk and ESP were left untouched.

## Explicit launcher recovery after exhaustion, 2026-10-18

The user matrix now replaces an exhausted launcher with the distribution's
LAUNCHER UNAVAILABLE / TRY AGAIN screen. The screen occupies the old launcher's
view; the app keeps its task, surfaces, current arrangement and unsaved work.
Clicking Try Again or pressing Enter with that view focused requests one new
stored-program read and admission. The acknowledgement precedes retirement of
the requesting screen. A successful replacement gets fresh layout and app
management authority, but no replenished automatic retry. Repeated failure
returns to the screen; its own exit or fault stays stopped.

The platform does not select distribution code. LAUNCH_IMAGES explicitly lends
existing executable grants to future store-based admissions, with none by default.
The startup owner lends its recovery executable to the user matrix and supplies
LAUNCHER_FALLBACK before CONFIGURE. The fallback has local interaction and a
connection-bound RETRY_LAUNCHER request, without storage, layout, task management
or descendant capacity. Kernel template admission retains its own code grant;
it does not grant capacity to spawn anything. Ordinary stored apps still receive
no executable grants. No kernel ABI, resource ceiling or stored schema changed.

Changing the owner's fallback policy cancels configured recovery, including a
queued request. Existing owner placement/storage/management revocation rules
also cancel recovery. This is an ordinary delegated recovery UI, not physical
human attestation or a protected global escape. It requires a live matrix and
its upstream services; live provider loss remains separate work.

Verification:

- Launcher cases passed across `launcher-recovery/` runs:
  `20261019T004036309505Z` completed recover (including cold boot) and clean-exit
  cases before a startup screenshot assertion failed. That assertion used the
  wrong glyph coordinate; corrected startup passed in `20261019T004252229040Z`.
  Recover exercised both pointer and keyboard retries, retained live layout and
  a dirty Notes draft, and saved/closed without restarting Notes. Startup faults
  proved each click permits exactly one additional attempt, without an automatic
  loop. `20261019T004341827568Z` completed owner-stop and owner-revocation cases
  before a fallback screenshot check caught the cursor covering its text.
  Moving the cursor away fixed the check; fallback clean-exit and fault cases
  passed in `20261019T004519997743Z`, with Notes still editable/saveable/closable.
  Successful subset runs confirmed the source fixture was unchanged.
- The initial fallback assertion incorrectly expected no code grant at all;
  kernel template admission intentionally includes the template's own code.
  The corrected runtime assertions verify no extra code and zero child capacity,
  plus denied storage/layout/management operations. No kernel change was needed.
  Read/admission refusal paths lead to the same fallback but were not separately
  fault-injected in this milestone.
- `management/20261019T004803880775Z` passed all three existing scenarios,
  including force-ending a hung app, both Cancel actions, stale/revoked authority,
  recovered launcher management, replacement apps and cold persistence. Notes
  now also asserts denial of fallback retry/policy and executable delegation on
  its interaction/storage endpoints, and absence of inherited executable grants.
- `desktop/20261019T004939308363Z` passed the normal F5 user/app flow and cold
  persistence, with the independent oracle checking all 237 records. The picker
  was ready 2.48 seconds after QMP connection on the first boot and 4.30 seconds
  on cold reboot. These timings exclude compilation/installation; they do not
  diagnose the owner's separate delayed-window report.
- `provider-startup/20261019T005026283061Z` passed `crash-once`: the shared
  distribution screen still handles failed provider readiness and explicit
  retry through to a usable session. `run.py --matrix-storage-test --release
  --timeout 180` passed recursive delegation, scoped launches, loss handling,
  owner stop/revocation and replacement through the shared admission/cleanup path.
- Workspace host tests, userspace and changed feature-set clippy passed. Layout
  (35 Omega/96 Rust edges), source formatting, Python compilation and diff checks
  passed. No kernel/architecture changes; ARM64 was not rerun.

Next: live provider-loss behavior that preserves app drafts. The trusted-chrome
owner question remains open. The user's interactive QEMU, disk and ESP remain
untouched; fault tests use disposable copies and separate boot media.

## Live storage loss with app drafts, 2026-10-18

F5 now polls replacement-provider readiness through
[replacement.rs](source-rs/distribution/desktop/session/persistence/replacement.rs).
It no longer blocks the startup owner for the provider's 30-second recovery
deadline. Initial login readiness and live replacement are distinct phases;
replacement success restores grants to the current user and its remembered
child realms without restarting the picker, launcher or app. The old provider
and its control endpoints are collected before a fresh attempt. Failure leaves
storage offline and the existing user processes alive. The three-replacement
session budget and six-second operation deadline are unchanged; the older lab
desktop keeps its synchronous replacement contract.

The new [live-provider fixture](tools/boot-harness-rs/provider_live_test.py) copies
a one-user default-layout image and uses the actual F5 composition. Opt-in owner
probes arm the storage service's existing fault gate only after a dirty note is
visible. No fault commands or probe features ship in the ordinary profile.

Verification:

- `provider-live/20261019T005500307323Z` passed precommit and postcommit stalls,
  including cold boot. Notes retained its draft and task, then F1 reconciliation
  correctly distinguished an uncommitted write from a committed lost reply.
  Subsequent editing/saving/closing worked and exact note pixels survived cold
  boot. Neither case restarted login; the source image was unchanged.
- `provider-live/20261019T005609024283Z` passed exhausted retries and blocked
  replacement readiness, both with cold boot and unchanged source media. Four
  precommit failures consumed exactly three replacement attempts. A provider
  that never answered readiness was collected at the existing deadline. Notes
  stayed editable while waiting and offline; Cancel retained its draft and a
  failed Save in the close dialog kept the dialog/process alive. Cold boot read
  the original saved note, proving the failed precommit writes were not published.
- `desktop/20261019T005819433872Z` passed the default F5 flow and cold persistence;
  the independent host oracle verified all 237 records, including the copied
  user, duplicate notes and saved color.
- `matrix-database/20261019T005930291094Z` passed the older recursive recovery
  lab: stalled-provider watchdog, uncertain CREATE reconciliation, precommit
  and lost-reply replacement, live sibling progress and cold persistence. The
  host oracle verified exact revisions `[4, 2]` and all 16 records on both boots.
- Workspace host tests, userspace clippy, shell and combined live-probe feature
  clippy, restart-budget policy tests, layout (35 Omega/96 Rust edges), formatting,
  Python compilation and diff checks passed. This changes only distribution
  supervision and its fixtures; no kernel, ABI or stored schema changes.

Remaining limits: this proves storage-provider failure, not display/input/root
replacement. Offline Notes keeps failed saves and close attempts visibly failed;
there is still no delegated live-session control to retry storage once automatic
recovery is exhausted. The existing protected-chrome owner question remains
separate from ordinary delegated recovery. Failure while restoring grants and
simultaneous provider/launcher death are not exhaustively injected here.
