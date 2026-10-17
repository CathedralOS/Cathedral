# Rust reference OS: tasks and handoff

Updated 2026-10-16 (local date). Implementation baseline: **`e8aa97d`**, pushed
to `main`, "Make F5 boot one desktop with an in-guest program launcher".
This document records the handoff after that change; check Git for later work.

## Start here

The user wants to boot Cathedral and use its functionality, without choosing
between disconnected host-side lab profiles. **F5 now boots one desktop with a
Programs menu. Preserve that as the default experience.**

From the repository root:

```powershell
python tools/boot-harness-rs/run.py --release --window
```

This is exactly what the single [VS Code launch entry](.vscode/launch.json) runs.
The composition is [distribution/profile.json](source-rs/distribution/profile.json).
It uses x86-64 QEMU/TCG and UEFI. Use the UEFI crate for firmware interfaces.

- Click **Notes** or **Colors** in Programs, then click the workspace to focus it.
- Notes is a **60-character ASCII scratchpad**. Type letters/digits/spaces;
  Enter inserts a newline, Left/Right move the cursor, Backspace deletes.
  F1 saves or reconciles an uncertain save; F2 reloads confirmed storage.
- Colors is transient; Enter changes its color. It has no storage grant.
- Close App closes the workspace. Selecting another program replaces the current
  app. Ordinary switching/closing preserves an unsaved note by refusing until
  the user saves or explicitly reloads it. There is one active workspace app.
- Menu Up/Down/Enter works while the menu has focus. There is no global shortcut
  or automatic workspace focus after launching yet.

Interactive notes persist in **`build/boot-harness-rs/cathedral-storage.raw`**.
Missing media is created; existing media is never automatically reformatted.
Older `notes-storage.raw`, `matrix-database-storage.raw`, `matrix-desktop-storage.raw`
and `storage.raw` belong to earlier fixtures. Do not overwrite them or silently
reinterpret their schemas. Automated tests use separate disposable images.

## Decisions and working expectations

- Cathedral is one monorepo: an independently usable **kernel**, a blessed common
  **platform** outside it, and one replaceable **distribution**. Desktop policy
  belongs to the distribution; capability enforcement belongs to its owning
  kernel/platform mechanism. Contracts live with the boundary owner.
- A matrix should use the same services recursively wherever possible. The root
  adapts physical input/output; children receive only their local connections,
  geometry and granted resources. No ambient service discovery or global IDs.
- Discoverability matters: a named entry shows sequencing/dispatch and leads
  into semantic subfolders. Rust `mod.rs` is plumbing only, beside the substantive
  named entrance. Avoid forwarding-only layers and unrelated folder dumping grounds.
- Keep the OS responsive and understandable. The user previously encountered
  flicker, lag and an apparently stuck cursor. Rendering/input changes need an
  actual QEMU interaction check, not just compilation or a serial success marker.
- Prior session direction was to commit and push progress to `main`. Check the
  working tree first and preserve unrelated work. `.codex/` was untracked and
  deliberately untouched at this handoff.
- Record unresolved owner-level semantics/trust choices in
  [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md). Ordinary implementation bugs and
  refactoring do not require an owner question. Existing design prose contains
  ambitions and unresolved conflicts; do not treat it all as implemented fact.

## What exists and where to enter

| Area | Current implementation | Reading entrance |
| --- | --- | --- |
| Kernel | UEFI exit, owned mappings/stacks, frame/heap allocation, interrupts/timer, preemption, isolated user processes, capability syscalls, IPC, admission/teardown, recursive resource funding and CPU ceilings | [Boot](source-rs/kernel/boot/uefi/main.rs), [user tasks](source-rs/kernel/core/user_tasks/user_tasks.rs) |
| Architecture | Shared core with x86-64 and an ARM64 QEMU portability proof; ISA and board responsibilities separated | [Architecture](source-rs/kernel/arch/README.md), [ARM64 bounds](source-rs/kernel/arch/isa/aarch64/README.md) |
| Matrix | Same service recursively owns children, local surfaces, focus, input, private storage delegation and bounded budgets | [Service loop](source-rs/platform/services/matrix/service/service.rs), [protocol](source-rs/platform/contracts/matrix.rs) |
| Display/input | Software compositor, retained rectangles and sealed bitmap objects, scoped pointer/focus routing, physical PS/2 input | [Display](source-rs/platform/services/display/main.rs), [input](source-rs/platform/services/input/main.rs) |
| Storage | Userspace ATA PIO transport, guarded volume and experimental redb-backed catalog/content work; transactions and recovery proofs | [Database provider](source-rs/platform/services/storage/database/main.rs), [catalog guide](source-rs/platform/libraries/object-store/database/README.md) |
| Desktop | Provider admission/supervision, program menu and one nested workspace | [Main](source-rs/distribution/desktop/main.rs), [session](source-rs/distribution/desktop/session/session.rs), [shell](source-rs/distribution/desktop/session/shell/shell.rs) |
| Notes | Local editing, atomic two-record catalog save, explicit reconciliation and sealed bitmap presentation | [Session](source-rs/distribution/applications/notes/session/session.rs) |

The [distribution map](source-rs/distribution/README.md) connects executable and
IPC seams. [STATUS.md](source-rs/STATUS.md) has broader mechanism/proof coverage;
[the Rust guide](source-rs/README.md) has ABI/build details.

The kernel is still bounded: twelve task accounts, a small boot-supplied code
catalogue, one CPU, static device grants and finite page/channel budgets. ARM64
is a headless portability proof, not the desktop. There is no production DMA/IOMMU
driver stack, SMP, GPU stack, audio stack or production DBFS. Do not equate the
redb experiment or BLAKE3 layout research with a finalized filesystem format.

## Latest change: preserve this context

`e8aa97d` promoted `distribution/lab/matrix` to **`distribution/desktop`**, renamed
its package to **`cathedral-desktop`**, and renamed its old `desktop/` child to
`session/`. Older proofs remain selectable features; they were not all turned into
apps. Old package paths/names in external notes will be stale.

The normal composition uses the **`shell` feature**, which includes `notes` and
`desktop`. The owner starts display (Launch 0), input (1), root matrix (2) and
database storage (3). Code catalogue entries are matrix (0), Notes (1), and the
desktop executable (2), reused for the menu and Colors. That last executable's
role 2 dispatches supplied app argument 3 to the menu, or 2 to transient Colors.
The seven-grant bound is why these are not four independent code entries.

Root slot 0 holds the workspace matrix; root slot 1 holds the menu. Workspace
slot 0 holds the active app. The menu is an isolated child with no launch/storage
authority. The owner polls its sequence/choice over the existing routed control
connection, validates the fixed choice, then sends matrix RUN/STOP/grant/placement
requests. Apps do not know their global coordinates or siblings. Notes gets root
0's private catalog; storage replacement regrants it only while Notes is open.

The earlier status/counter boot survives as
[profiles/legacy.json](tools/boot-harness-rs/profiles/legacy.json). Existing
`--input-test`, `--screenshot`, `--recovery-test` and `--storage-test` flags select
that fixture; they do **not** test the new menu. `--smoke` still uses the default
profile's `user_programs`, not its desktop startup graph.

## Next work, in recommended order

These are proposed follow-ups, not claims that the owner has approved every
future feature. The documentation handoff itself leaves no implementation running.

- [ ] **Harden desktop app lifecycle before adding more apps.** Cover close/switch
  during a slow save, app crash, startup failure and provider loss. Keep the menu
  usable and report failure visibly. Current shell admission paths assert success;
  an app failure is not yet a complete recoverable desktop workflow.
  In particular, the Notes dirty check uses a routed owner HEALTH request.
  [Forwarded control](source-rs/platform/services/matrix/service/children/control.rs)
  retires and stops a child after a failed/timed-out exchange. Investigate whether
  switching during stalled storage can therefore kill a draft-bearing app. This
  is a **code-review concern, not a reproduced failure**; the normal draft guard
  passed, and the existing notes-recovery fixture does not exercise menu switching.
  Do not solve it by blindly extending every timeout or automatically replaying
  uncertain writes. Add an adversarial test for the actual failure being fixed.
- [ ] **Make the next storage milestone usable in the same desktop.** Expose a
  small document/object workflow through matrix-scoped bulk-content capabilities,
  backed by existing storage work. A user should open, edit, save and reopen data
  from F5. The current 60-character note uses the tiny catalog protocol; increasing
  that wire limit is not a general document or filesystem design.
- [ ] **Keep navigation clear as the promoted desktop grows.** Its historical
  headless scenarios and recovery probes still coexist with normal session code.
  Separate by actual responsibility when touching them; keep normal startup easy
  to follow. Avoid another large restructuring solely to move filenames.
- [ ] **Choose the next kernel/device milestone deliberately.** Candidates are a
  second userspace MMIO/IRQ device proof and extending admission beyond the fixed
  boot catalogue. Keep ISA/device mechanism in kernel owners and desktop policy
  outside. DMA needs a separate buffer ownership/isolation design; do not imply
  that granting MMIO alone makes an arbitrary DMA-capable driver safe.

An intermittent ATA timeout is recorded in the existing implementation map and
remains undiagnosed. The latest desktop/recovery tests below passed. Retain logs
and distinguish a reproduced transport failure from a guessed cause if it recurs.
SIMD rendering, general multi-window management and production DBFS choices are
not prerequisites for the immediate lifecycle work.

## Validation and last evidence

Run Cargo commands **inside `source-rs/`** so its toolchain, aliases and linker
configuration apply:

```powershell
cargo test --locked
cargo check-userspace
cargo clippy --locked --package cathedral-desktop --target x86_64-unknown-none --features shell -- -D warnings
```

From the repository root, select checks appropriate to the change:

```powershell
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/matrix_database_test.py --release --notes-recovery
python tools/source-layout/check.py
python tools/source-layout/check_test.py
python tools/source-layout/format-rust.py --check
```

`desktop_test.py` builds the **same default composition as F5**. It injects mouse
and keyboard input, compares exact note glyphs/status, refuses dirty switches,
saves/reloads, repeatedly replaces apps, and cold-boots the same disposable disk.
The recovery fixture separately checks two private catalogs, lost acknowledgement,
draft reconciliation and an independent host database oracle. Extend the default
desktop test when integrating user-visible functionality; keep destructive proofs
on disposable disks. Kernel/arch changes need the relevant smoke/ARM64 suites
documented in their guides, not merely these UI tests.

Passed before the handoff at `e8aa97d`: all three Cargo commands above, layout
checks and formatting, `database_image_test.py`, default desktop test,
notes-recovery fixture, and preserved `run.py --input-test --release`.
No kernel smoke or ARM64 rerun was claimed for that desktop-only change.

Local evidence (ignored build artifacts; regenerate on another machine):

- `build/boot-harness-rs/desktop/20261017T040852963620Z/`: menu, repeated app
  replacement, exact note pixels and cold restore; serial logs and screenshots.
- `build/boot-harness-rs/matrix-database/20261017T041056161085Z/`: notes recovery,
  cold restore, exact 16-record host verification, timing and ATA samples.

When handing off again, update the baseline commit, next concrete task, known
failures and verification scope here. Keep durable subsystem detail in its owner
docs, and avoid leaving two conflicting descriptions of the default OS boot.
