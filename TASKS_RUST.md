# Rust reference OS: tasks and handoff

Updated 2026-10-18 (local date). Current milestone: authorized live matrix layout
and rendering invalidation recovery, following saved configuration and named realms. F5 uses
these mechanisms; explicit migration preserves existing stock users and data.
F5 now compacts existing media before changed program installations, fixing
the retained-disk upgrade failure described at the end of this handoff.
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
- [ ] **Keep app lifecycle honest as apps grow.** Startup failure, app crash and
  provider loss while an app holds work still end in assertions in places. A
  program that cannot be read from the realm answers REFUSED in the chooser
  but the desktop still asserts on some root startup operations. User launcher
  lookup failure now returns to the picker. Drawing invalidation during live
  layout now rebuilds views without losing drafts. Add an adversarial test for
  each failure actually fixed. App-owned close with unsaved-work choices now works;
  owner recovery from an unresponsive app and clearer focus feedback remain.
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
focus. It edits strip width (192–320 pixels) and outer spacing (8–40 pixels).
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
