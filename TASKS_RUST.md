# Rust reference OS: tasks and handoff

Updated 2026-10-18 (local date). Implementation baseline: declared service
imports and optional Notes persistence at **`8d1972a`**, followed by readable
chooser labels at **`787344e`**, existing-user label upgrade at **`4f1c6c8`**,
app-owned close at **`a40ed23`**, and the user tiles/absolute input milestone
below. Store-based
admission validates the program's required/optional interfaces against owner
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
launchers and Notes/Colors in existing users, preserving their saved data and
user matrices. The app-close upgrade saves
`cathedral-storage.raw.before-app-close.raw` before changing user copies; the
earlier label-only backup remains separate. Customized/unrecognized programs
stay intact.
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

The intended shape, not yet built: boot into the boot matrix, which runs the
user selection and creation software; a user matrix carries an installed
windower or chrome. Today the owner program stands in for the boot matrix and
the chooser stands in for both the user software and the chrome.

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
nothing. The owner reads what a slot holds with `STORAGE_BINDING` and restores
exactly that after replacing a provider.

A user is a subtree copy of the template user: a matrix executable with the
chrome, Notes and Colors inside it. The desktop furnishes every user matrix the
same way it furnishes the root: the chrome in the first slot over the whole
realm, a standing policy for the second. `desktop_test.py` drives this from F5
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
| Storage | Userspace ATA PIO transport, guarded volume, redb-backed object store with a tree of numbered objects up to 256 KiB, shared content, copies, subtree copies, pins and metadata-only bounded collection; one object protocol with scope enforcement | [Provider](source-rs/platform/services/storage/main.rs), [store](source-rs/platform/libraries/database-objects/README.md), [contract](source-rs/platform/contracts/storage.rs) |
| Desktop | Provider admission/supervision, chooser over users, furnishing of user matrices, all programs from the store; probe desktop under `lab/` on the boot catalogue without the shell feature | [Main](source-rs/distribution/desktop/main.rs), [session](source-rs/distribution/desktop/session/session.rs), [shell](source-rs/distribution/desktop/session/shell/shell.rs), [lab](source-rs/distribution/desktop/lab/desktop.rs) |
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
Colors. Identical bytes share content, so the second chooser costs three small
records. The shell names only the first two by number
([shell.rs](source-rs/distribution/desktop/session/shell/shell.rs)); everything
below is found by listing. Roles come from each executable's entry: a matrix
takes 0, the desktop executable's 2 runs Colors with argument 2, Notes and the
chooser take 2.

Root slot 0 holds the chooser, shown only the users folder; root slot 1 is
opened by a standing policy and holds whatever the chooser launches, a user
matrix. The desktop notices it by polling `STORAGE_BINDING` on that slot and
furnishes it: its chrome in slot 0 over the whole user realm, slot 1 opened
for what the chrome launches. A user made by NEW is a subtree copy of the
template, so user 7 holds chrome 8, Notes 9 and Colors 10; their notes and
color nest under 9 and 10. Apps do not know their global coordinates or
siblings. Storage replacement regrants the chain explicitly from the shell's
numbers and the bindings it observed.

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

These are proposed follow-ups. The owner clarified that the boot environment
orchestrates privileged providers and delegates services; it need not be one
matrix executable. See the clarified startup/policy question in OWNER_QUESTIONS.

- [ ] **Persist startup composition and matrix policy.** Keep the initial owner
  as the composition root while choosing where policy lives and who may edit
  it. Do not merge drivers/providers into the matrix or conflate this with
  authentication. Declared service imports and launch-time binding are now the
  first bounded implementation of the supplied-environment model.
- [ ] **Give objects kinds and editable names.** The chooser now shows program
  display titles and user ordinals, while the
  user picker copies its first entry as the template. The user launcher now
  toggles storage instead of copying chrome, but object roles remain implicit.
  Resolve template selection and readable program labels without relying on
  object ordering; authentication remains a separate decision.
- [ ] **Keep app lifecycle honest as apps grow.** Startup failure, app crash and
  provider loss while an app holds work still end in assertions in places. A
  program that cannot be read from the realm answers REFUSED in the chooser
  but the desktop asserts on its own launches. Add an adversarial test for each
  failure actually fixed. App-owned close with unsaved-work choices now works;
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

**Outstanding storage finding:** With the older
`desktop/20261017T232223657195Z/guest.raw` fixture upgraded, adding USER 2 twice
hit a storage deadline and provider replacement; the picker displayed TRY AGAIN.
Existing-user selection/readback passed before that extra stress. Artifacts:
`labels-upgrade/20261018T174631398112Z/` and `20261018T174825636437Z/`.
The newer fixture's equivalent two extra copies passed. Cause is unproven;
do not claim this is the previously recorded ATA timeout without tracing it.
No retry of an uncertain copy was added. The original chooser already lacks a
reconcile/reload action after uncertain creation; investigate that alongside the
storage stall before adding broader account controls. The new capacity check
is opt-in so its distinct purpose is explicit.

The old F5 test's storage-toggle/temporary-Notes steps were removed with that
UI. Earlier evidence remains above; service import policy host tests still run,
but this milestone does not claim new end-to-end optional-storage UI coverage.
