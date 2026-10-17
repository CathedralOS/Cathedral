# Rust reference OS: tasks and handoff

Updated 2026-10-17 (local date). Implementation baseline: the commit after
`39b935b` ("Delete the legacy status/counter composition and its sector
catalog") that adds worlds, scoped delegation and subtree copies. Check Git for
later work.

## Start here

F5 boots one desktop with a Programs menu. **Preserve that as the default
experience.** From the repository root:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

This is exactly what the single [VS Code launch entry](.vscode/launch.json) runs.
The composition is [distribution/profile.json](source-rs/distribution/profile.json).
It uses x86-64 QEMU/TCG and UEFI.

- Click **Notes** or **Colors** in Programs, then click the workspace to focus it.
- Notes keeps up to eight notes of 240 ASCII characters in its world.
  Type letters/digits/spaces; Enter inserts a newline, Left/Right move the
  cursor, Backspace deletes. F1 saves or reconciles an uncertain save; F2
  reloads confirmed storage; F3 duplicates the saved note as a new note that
  shares its content; F4 switches to the next note. A bar below the text shows
  the notes, with the current one bright.
- Colors is transient; Enter changes its color. It has no storage grant.
- Close App closes the workspace. Selecting another program replaces the current
  app. **Clone World** copies the current world with every note in it, by
  reference, and reopens the program there; **Next World** cycles through the
  worlds. The menu's last line says which world of how many is open. Switching
  or closing refuses while a note has an unsaved draft, and leaves a note open
  rather than killing it when the app is too busy to answer.
- Menu Up/Down/Enter works while the menu has focus.

Interactive notes persist in **`build/boot-harness-rs/cathedral-storage.raw`**.
Missing media is created; existing media is never automatically reformatted.
An image written before worlds existed still holds the earlier object table
under its old name; the desktop ignores it and starts a fresh world, so delete
the file to reclaim the space. `notes-storage.raw` and
`matrix-database-storage.raw` belong to the notes and probe desktop fixtures.
Automated tests use separate disposable images.

## What the storage milestone established

The thesis is that cheap file copies make cheap world copies. The booted OS now
has both halves. A root holds many numbered objects arranged as a tree; chunks
and manifests are shared within the root; a copy writes a head, an event and an
edge; a subtree copy does that for every object beneath the source. A matrix
grants a child a subtree rather than a root, stamps that scope on every request
it forwards, and the provider enforces every stamp, so an app sees its home and
nothing beside it. The desktop keeps worlds as top-level objects, grants the
workspace one world, and Clone World is the store's subtree copy. Notes
exercises that from F5, and an independent host model verifies two worlds of
notes sharing manifests after a cold boot. One object protocol serves the
provider, the matrix forwards it with sealed attachments both ways, and apps use
a runtime scope that reports their home.

## Decisions and working expectations

- Cathedral is one monorepo: an independently usable **kernel**, a blessed common
  **platform** outside it, and one replaceable **distribution**. Desktop policy
  belongs to the distribution; capability enforcement belongs to its owning
  kernel/platform mechanism. Contracts live with the boundary owner.
- A matrix is an ordinary program. The instance handed provider links is the
  storage root and the one handed display/input endpoints adapts hardware;
  every other instance only forwards and attenuates. Nothing branches on depth
  or on an argument word.
- Delegation narrows and never widens. A grant names a scope inside what the
  granting matrix holds; the provider, not the matrices, is what makes the chain
  binding. An owner prepares what it grants on the matrix's own authority
  (`STORAGE_REQUEST`), never by reaching past its child.
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
| Kernel | UEFI exit, owned mappings/stacks, frame/heap allocation, interrupts/timer, preemption, isolated user processes, capability syscalls, IPC, admission/teardown, recursive resource funding and CPU ceilings; 128 private/reader pages per task | [Boot](source-rs/kernel/boot/uefi/main.rs), [user tasks](source-rs/kernel/core/user_tasks/user_tasks.rs) |
| Architecture | Shared core with x86-64 and an ARM64 QEMU portability proof; ISA and board responsibilities separated | [Architecture](source-rs/kernel/arch/README.md), [ARM64 bounds](source-rs/kernel/arch/isa/aarch64/README.md) |
| Matrix | Same service recursively owns children, local surfaces, focus, input, subtree delegation with scope stamps and attachments, owner requests on its own authority, bounded budgets; counts unanswered child requests instead of killing a slow child | [Service loop](source-rs/platform/services/matrix/service/service.rs), [protocol](source-rs/platform/contracts/matrix.rs), [storage forwarding](source-rs/platform/services/matrix/service/parent/storage/storage.rs) |
| Display/input | Software compositor, retained rectangles and sealed bitmap objects, scoped pointer/focus routing, physical PS/2 input | [Display](source-rs/platform/services/display/main.rs), [input](source-rs/platform/services/input/main.rs) |
| Storage | Userspace ATA PIO transport, guarded volume, redb-backed object store with a tree of numbered objects, shared content, copies, subtree copies, pins and bounded collection; one object protocol with scope enforcement | [Provider](source-rs/platform/services/storage/main.rs), [store](source-rs/platform/libraries/database-objects/README.md), [contract](source-rs/platform/contracts/storage.rs) |
| Desktop | Provider admission/supervision, worlds, program menu and one nested workspace; probe desktop under `lab/` without the shell feature | [Main](source-rs/distribution/desktop/main.rs), [session](source-rs/distribution/desktop/session/session.rs), [worlds](source-rs/distribution/desktop/session/worlds.rs), [shell](source-rs/distribution/desktop/session/shell/shell.rs), [lab](source-rs/distribution/desktop/lab/desktop.rs) |
| Notes | Whole-document publication under the binding's home, bulk reads, duplicate through the store's copy, note switching, explicit reconciliation, sealed bitmap presentation | [Session](source-rs/distribution/applications/notes/session/session.rs), [storage](source-rs/distribution/applications/notes/session/storage.rs) |

The [distribution map](source-rs/distribution/README.md) connects executable and
IPC seams. [STATUS.md](source-rs/STATUS.md) has broader mechanism/proof coverage;
[the Rust guide](source-rs/README.md) has ABI/build details.

The kernel is still bounded: twelve task accounts, a small boot-supplied code
catalogue, one CPU, static device grants and finite page/channel budgets. ARM64
is a headless portability proof, not the desktop. There is no production DMA/IOMMU
driver stack, SMP, GPU stack, audio stack or production DBFS. Do not equate the
object store or BLAKE3 layout research with a finalized filesystem format.

## Composition facts worth knowing

The owner starts display (Launch 0), input (1), root matrix (2) and object
storage (3). Code catalogue entries are matrix (0), Notes (1) and the desktop
executable (2), reused for the menu and Colors. That last executable's role 2
dispatches supplied app argument 3 to the menu, or 2 to Colors. The seven-grant
bound is why these are not four independent code entries.

Root slot 0 holds the workspace matrix; root slot 1 holds the menu. Workspace
slot 0 holds the active app. The menu is an isolated child with no launch/storage
authority. The owner polls its sequence/choice over the existing routed control
connection, validates the fixed choice, then sends matrix RUN/STOP/grant/placement
requests. Apps do not know their global coordinates or siblings. Worlds are
top-level objects of provider root 0, listed and created by the owner at
startup; the workspace is granted the current world as its scope and Notes gets
the whole of that, so its home is the world. Storage replacement regrants it only
while Notes is open, and a grant after replacement re-establishes the root's
provider connection. Changing worlds stops the app and reopens it under the
new grant; a clone happens before anything closes, so a refused clone leaves the
app open.

Page budgets are sized for sealed bitmaps held at every matrix level with old
and new present during replacement: the root matrix has 128 private and 128
reader pages, the workspace 40/40, Notes 32/4, the menu 40/0 (two 20-page
bitmaps of seven text rows), Colors 2/0, the provider 64/8, the display 96
reader pages. The kernel's per-task bound is 128 of each.

A request carries three scope words, so three narrowing grants can stack on one
path; a fourth is refused. Each scoped request costs the provider one ancestry
walk per stamp. Both are lab bounds, not the design's.

`--smoke` uses the default profile's `user_programs`, not its desktop startup
graph; the desktop is tested by `desktop_test.py`.

## Next work, in recommended order

These are proposed follow-ups, not claims that the owner has approved every
future feature.

- [ ] **Keep app lifecycle honest as apps grow.** Startup failure, app crash and
  provider loss while an app holds work still end in assertions in places,
  including a desktop that cannot list its worlds at startup. A refused control
  probe now leaves a note open and a refused clone reports STORAGE FAILED; make
  the menu say why in each remaining case and offer an explicit stop. Add an
  adversarial test for each failure actually fixed.
- [ ] **Let a world name things.** Objects have parents and numbers but no
  names, and a world cannot be deleted while it holds notes. Decide how a world
  is labelled and retired without resolving the identity/alias owner question by
  implication; the menu's "WORLD n OF m" is a position, not an identity.
- [ ] **Choose the next kernel/device milestone deliberately.** Candidates are a
  second userspace MMIO/IRQ device proof and extending admission beyond the fixed
  boot catalogue; executables from the store can now live in a world's subtree.
  DMA needs a separate buffer ownership/isolation design.

An intermittent ATA timeout is recorded in the implementation map and remains
undiagnosed. Retain logs and distinguish a reproduced transport failure from a
guessed cause if it recurs.

## Validation and last evidence

Run Cargo commands **inside `source-rs/`** so its toolchain, aliases and linker
configuration apply:

```powershell
cargo test --locked
cargo test --locked -p cathedral-database-objects --features host
cargo check-userspace
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
python tools/source-layout/check.py
python tools/source-layout/check_test.py
python tools/source-layout/format-rust.py --check
```

`desktop_test.py` builds the **same default composition as F5**. It injects mouse
and keyboard input, compares exact note glyphs, status, world line and note bar,
refuses dirty switches and clones, saves, duplicates, edits the copy, switches
back, reloads, repeatedly replaces apps, clones the world, edits the clone's
note, returns to the original, cold-boots the same disposable disk, checks both
worlds, and has the host lab verify the retained records on a copy of the image.
Kernel/arch changes need the smoke and ARM64 suites documented in their guides:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
python tools/boot-harness-rs/arm64/run.py --release
```

Passed before this handoff, on the worlds commit: the four Cargo commands above
plus clippy of every desktop, notes, matrix and storage feature set the profiles
use, both layout checks and formatting, every harness command listed above, the
x86 release smoke suite, the ARM64 release proof, `--matrix-test`,
`--admission-test` and `--ipc-test`. Oracles verified the F5 image (two worlds
of two notes sharing manifests, exact edges, empty second root), the probe
desktop images at object revisions [2, 2] and [4, 2], the notes images, and
both objects proofs.

When handing off again, update the baseline commit, next concrete task, known
failures and verification scope here. Keep durable subsystem detail in its owner
docs, and avoid leaving two conflicting descriptions of the default OS boot.
