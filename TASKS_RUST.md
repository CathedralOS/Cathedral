# Rust reference OS: tasks and handoff

Updated 2026-10-17 (local date). Implementation baseline: the commit after
`1e23ca8` ("Admit executables from sealed memory objects") that installs the
desktop's programs into the store and launches them from the matrix's realm.
Check Git for later work.

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
- Notes keeps up to eight notes of 240 ASCII characters in its realm.
  Type letters/digits/spaces; Enter inserts a newline, Left/Right move the
  cursor, Backspace deletes. F1 saves or reconciles an uncertain save; F2
  reloads confirmed storage; F3 duplicates the saved note as a new note that
  shares its content; F4 switches to the next note. A bar below the text shows
  the notes, with the current one bright.
- Colors changes its color on Enter and keeps it, because its state nests under
  its own program like everything else's.
- Close App closes the workspace. Selecting another program replaces the current
  app. Switching or closing refuses while a note has an unsaved draft, and leaves
  a note open rather than killing it when the app is too busy to answer.
- Menu Up/Down/Enter works while the menu has focus.

Interactive state persists in **`build/boot-harness-rs/cathedral-storage.raw`**.
Missing media is created; existing media is never automatically reformatted.
The programs are installed into the image on first launch and republished as
new revisions when their bytes change; the sidecar `cathedral-storage.raw.programs`
records what is installed. An image from before programs lived in the store
holds notes at the numbers the programs need; the installer refuses to
overwrite anything that is not a program, so delete that file to start clean.
`notes-storage.raw` and `matrix-database-storage.raw` belong to the notes and
probe desktop fixtures. Automated tests use separate disposable images.

## The architecture, in the owner's words

Realms are the attenuated filesystem. Matrices are programs that attenuate,
heavily or not. If the boot matrix is correct, holds every capability, can
launch programs, and the filesystem is database-backed and attenuable, the rest
of the matrix falls out for almost free. The real logic is then that a launched
program gets its own subtree for free: everything nested under the program's
object, because files are folders in this database.

## What the store milestones established

The thesis is that cheap file copies make cheap world copies. The booted OS has
the mechanism: a root holds numbered objects arranged as a tree; chunks and
manifests are shared within the root; a copy writes a head, an event and an
edge; a subtree copy does that for every object beneath the source. A matrix
grants a child a subtree rather than a root, stamps that scope on every request
it forwards, and the provider enforces every stamp, so a program sees its realm
and nothing beside it.

Programs now live in the store. The harness installs the profile's `programs`
into the notes root as objects, trimmed to their loadable bytes, nested as the
profile says. A matrix runs one with `RUN_OBJECT`: it reads the object on its
own binding, the kernel admits the sealed reply as the child's code, and the
child is granted that object's subtree, whose root the provider keeps read-only
to its holder. The workspace matrix's subtree holds its copies of Notes and
Colors, their notes and color nest under those copies, and the launcher is a
second copy of the desktop executable beside the workspace. The desktop holds
no storage authority and names programs by object number. `desktop_test.py`
drives this from F5 and an independent host model verifies the whole layout;
the headless storage fixture runs an installed program and checks its realm.

The kernel's contribution is one syscall: a sealed object a task can read
becomes an executable grant, consumed by one reservation and admitted once,
with the loader reading page-wise through the identity map. The kernel still
knows no matrix, realm or store.

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
  own realm.
- Use the design's vocabulary. A realm is a subtree held through a binding;
  a world is a matrix's realm in the wiki's sense (the object a matrix owns,
  with its state beneath it), which the Rust OS does not yet implement. Do not
  coin desktop-level names for storage structure.
- Boot launches and storage-less fixtures keep the boot catalogue, because the
  provider itself must come from boot. The matrix's RUN by catalogue image
  serves those compositions; RUN_OBJECT serves the store.
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
| Kernel | UEFI exit, owned mappings/stacks, frame/heap allocation, interrupts/timer, preemption, isolated user processes, capability syscalls, IPC, admission/teardown from boot templates or sealed objects, recursive resource funding and CPU ceilings; 128 private/reader pages per task | [Boot](source-rs/kernel/boot/uefi/main.rs), [user tasks](source-rs/kernel/core/user_tasks/user_tasks.rs), [code grants](source-rs/kernel/core/user_tasks/admission/pool/README.md) |
| Architecture | Shared core with x86-64 and an ARM64 QEMU portability proof; ISA and board responsibilities separated | [Architecture](source-rs/kernel/arch/README.md), [ARM64 bounds](source-rs/kernel/arch/isa/aarch64/README.md) |
| Matrix | Same service recursively owns children, local surfaces, focus, input, realm delegation with scope stamps and attachments, programs run from its realm, bounded budgets; counts unanswered child requests instead of killing a slow child | [Service loop](source-rs/platform/services/matrix/service/service.rs), [protocol](source-rs/platform/contracts/matrix.rs), [storage forwarding](source-rs/platform/services/matrix/service/parent/storage/storage.rs) |
| Display/input | Software compositor, retained rectangles and sealed bitmap objects, scoped pointer/focus routing, physical PS/2 input | [Display](source-rs/platform/services/display/main.rs), [input](source-rs/platform/services/input/main.rs) |
| Storage | Userspace ATA PIO transport, guarded volume, redb-backed object store with a tree of numbered objects up to 256 KiB, shared content, copies, subtree copies, pins and metadata-only bounded collection; one object protocol with scope enforcement | [Provider](source-rs/platform/services/storage/main.rs), [store](source-rs/platform/libraries/database-objects/README.md), [contract](source-rs/platform/contracts/storage.rs) |
| Desktop | Provider admission/supervision, program menu and one nested workspace, all programs from the store; probe desktop under `lab/` on the boot catalogue without the shell feature | [Main](source-rs/distribution/desktop/main.rs), [session](source-rs/distribution/desktop/session/session.rs), [shell](source-rs/distribution/desktop/session/shell/shell.rs), [lab](source-rs/distribution/desktop/lab/desktop.rs) |
| Notes | Whole-document publication in the binding's realm, bulk reads, duplicate through the store's copy, note switching, explicit reconciliation, sealed bitmap presentation | [Session](source-rs/distribution/applications/notes/session/session.rs), [storage](source-rs/distribution/applications/notes/session/storage.rs) |
| Harness | Builds, trims and installs programs; formats and seeds images; host oracles over copies of guest images | [Harness guide](tools/boot-harness-rs/README.md), [seeder](tools/storage-lab/engines/volume/seed.rs) |

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
storage (3) from boot. The programs are objects 1 to 4 of the notes root: the
workspace matrix; inside it Notes and the color app; and beside it the
launcher, a second copy of the desktop executable. Identical bytes share
content, so the second copy costs three small records. The shell names them by
these numbers ([shell.rs](source-rs/distribution/desktop/session/shell/shell.rs))
in the order of the profile's `programs`; roles come from each executable's
entry (a matrix takes 0, the desktop executable's 2 runs the supplied app with
argument 3 for the launcher and 2 for Colors, Notes takes 2).

Root slot 0 holds the workspace matrix; root slot 1 holds the menu. Workspace
slot 0 holds the active app. The menu is an isolated child with no launch
authority. The owner polls its sequence/choice over the existing routed control
connection, validates the fixed choice, then sends matrix RUN_OBJECT/STOP and
placement requests; the matrix grants each child its program's subtree itself.
Apps do not know their global coordinates or siblings. Storage replacement
regrants the chain explicitly: the workspace its program, and the open program
its own, from the shell's numbers.

Page budgets are sized for sealed bitmaps held at every matrix level with old
and new present during replacement, plus the largest program a matrix reads
to run it: the root matrix has 128 private and 128 reader pages, the workspace
40/72, Notes 32/4, the menu 24/0, Colors 2/0, the provider 128/64, the display
96 reader pages. The kernel's per-task bound is 128 of each. Objects hold at
most 256 KiB (64 pages); the desktop executable trims to about 110 KiB.

A request carries three scope words, so three narrowing grants can stack on one
path; a fourth is refused. Each scoped request costs the provider one ancestry
walk per stamp. Collection before every write scans only metadata and the
key-only chunk index; reading chunk bytes there cost 1.6 s per note save with
programs in the root and was the one stall found. All are lab bounds.

`--smoke` uses the default profile's `user_programs`, not its desktop startup
graph; the desktop is tested by `desktop_test.py`.

## Next work, in recommended order

These are proposed follow-ups, not claims that the owner has approved every
future feature.

- [ ] **Make the boot matrix the privileged program.** Today the initial
  program is the desktop owner, which admits a root matrix beside itself and
  drives it over a control connection. In the owner's framing those are one
  program: boot hands everything to the boot matrix, and the shell is a program
  it hosts from its realm. Collapsing the split is a distribution decision; it
  also decides what a matrix persists for itself, which is where the wiki's
  "a Matrix is an object owning its world-realm" lands. Do not resolve that by
  implication.
- [ ] **Keep app lifecycle honest as apps grow.** Startup failure, app crash and
  provider loss while an app holds work still end in assertions in places. A
  refused control probe now leaves a note open; make the menu say why and offer
  an explicit stop. A program that cannot be read from the realm is reported
  to the owner but the desktop asserts on it. Add an adversarial test for each
  failure actually fixed.
- [ ] **Let the lab desktops run from the store too.** The probe desktop and
  the notes fixtures still lend the boot catalogue, which keeps the admission
  and funding proofs on their original path. When those proofs have store
  equivalents, retire RUN by catalogue image from the matrix.
- [ ] **Choose the next kernel/device milestone deliberately.** A second
  userspace MMIO/IRQ device proof is the candidate. DMA needs a separate buffer
  ownership/isolation design.

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
python tools/source-layout/check.py
python tools/source-layout/check_test.py
python tools/source-layout/format-rust.py --check
```

`desktop_test.py` builds the **same default composition as F5**, installs the
same programs into a disposable image, injects mouse and keyboard input,
compares exact note glyphs, status and note bar, refuses dirty switches, saves,
duplicates, edits the copy, switches back, reloads, repeatedly replaces apps,
checks Colors keeps its color, cold-boots the same disposable disk, and has the
host lab verify the retained records on a copy of the image. `--trace-storage`
adds provider phase timestamps to its serial log. Kernel/arch changes need the
smoke and ARM64 suites documented in their guides:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
python tools/boot-harness-rs/arm64/run.py --release
```

Passed before this handoff, on the programs commit: the five Cargo commands
above plus clippy of every desktop, notes, matrix and storage feature set the
profiles use, both layout checks and formatting, every harness command listed
above, the x86 release smoke suite, the ARM64 release proof, `--matrix-test`,
`--admission-test` and `--ipc-test`. Oracles verified the F5 image (four
installed programs, two notes sharing one manifest under Notes, the color under
Colors, empty second root), the probe desktop images at object revisions
[2, 2] and [4, 2], the notes images, and both objects proofs.

When handing off again, update the baseline commit, next concrete task, known
failures and verification scope here. Keep durable subsystem detail in its owner
docs, and avoid leaving two conflicting descriptions of the default OS boot.
