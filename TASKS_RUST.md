# Rust reference OS: tasks and handoff

Owner update 2026-10-23: repeated launches of the same program over one realm
are accepted; concurrency or refusal is the app's policy. See the
[accepted contract](wiki/spec/execution/components_and_matrices.md#multiple-instances-over-one-realm).
The question is removed from OWNER_QUESTIONS.md; older entries calling it blocked
are superseded. The owner has now ratified the
[coordination and checked shared-memory IPC operation](wiki/spec/communication/ipc.md)
for the shipped OS/default matrix: scoped registration and introductions,
independent executable/path/secret constraints, app-controlled refusal and
protocol-enforced payload custody. The wire protocol and implementations remain
outstanding. This supersedes the earlier standalone named-lock proposal.

Owner clarification 2026-10-23: coordination controls introductions, not ongoing
authority over established peers. Services enforce ongoing permission/cancellation
in their own protocols; the first participant gains no implicit power to revoke
peer mappings or independent rights. See the
[accepted boundary](wiki/spec/communication/ipc.md#established-connections-and-service-authority).
Connection closure must preserve live memory-view obligations. Exact queued-message,
pending-transfer and blocked-wait outcomes still need transport contracts.

Owner clarification 2026-10-23: a failed or terminated matrix takes down its
entire owned subtree. No surviving descendant adoption; restart admits new
instances and may reopen authorized durable state. Independently owned external
providers are outside that subtree. See the
[matrix lifetime contract](wiki/spec/execution/components_and_matrices.md#matrix-failure-and-subtree-lifetime).
The Rust lab already cancels owned children on supervisor exit/fault; this
documentation ratification does not add runtime behavior or fresh QEMU evidence.
Budget refund routing and external-operation lifetimes remain separate contracts.

Owner clarification 2026-10-23: execution spends the executing program's budget
within its ancestor ceilings, including when it serves another program. Services
and matrices have no special scheduling status; request quotas and backpressure
belong to their own policies. The
[execution-budget contract](wiki/spec/resources/execution_budgets.md) closes CPU
sponsorship as an owner blocker. Caller-funded execution is deferred pending a
concrete need, not a required coordination milestone. Historical entries calling
that baseline unresolved are superseded; other resource sponsorship is unchanged.
This is a documentation ratification of the current lab direction, not a new
runtime implementation or fresh scheduling verification.

Owner clarification 2026-10-23: [hardware-separated programs](wiki/spec/execution/isolation.md)
are the current isolation architecture for apps, matrices and platform providers.
PCC-based single-address-space isolation is off the roadmap, not a future proof
threshold to implement. Reconsideration requires CHERI support and a separate
contract. Explicit shared IPC mappings remain supported. The old default-isolation
question is closed; optional PCID validation coverage remains separate and deferred.
This documentation decision introduces no runtime changes or fresh QEMU evidence.

Owner clarification 2026-10-23: default chrome/background programs come from the
system realm and everyday selections are replaceable. The outer system retains
protected recovery chrome: a physical OS-key long press brings it back even when
custom chrome or its matrix fails. Inner matrices need not include this escape
hatch; they may receive a system-realm copy and choose their own overrides.
See the [accepted surface contract](wiki/spec/human_surface/compositor_and_seat.md).
Override slot format and recovery implementation are not frozen. The Rust lab
does not yet implement the protected long-press path; current child close and
launcher restart are not that guarantee. No runtime behavior changed here.

Next implementation slice: add funded coordination acquisition to the existing
broker and prove simultaneous first registration, rejected identity spoofing,
constraint combinations, accepted/refused introductions and dead-holder cleanup.
Exercise duplicate Nested launches through F5 while unrelated apps keep running.
Build on existing endpoint transfer first; reusable shared-memory mailbox access
needs its checked/admitted provider and lifecycle before exposing stable leases.
Do not label the present Rust copied channels or sealed attachments as that
complete implementation. Path rename semantics, transport commit/closure rules
and general capability revocation remain outside the accepted partial contract.

Updated 2026-10-22 (local date). Current milestone: **the hop on hardware, and
global shared leaves**, on baseline `f8bab679` (where a forwarded request
spends its time). The harness runs the guest under the Windows hypervisor with
`--accel whpx` as well as under TCG: 32 bare round trips between two tasks cost
118 to 144 µs there against 3,140 to 4,824 under TCG, and the hop between
matrices sits inside the trace line's own noise. The seven supervisor ranges
every root shares are global now, so the kernel's translations survive the two
root switches of a trap: 15 percent off the round trip on hardware, nothing
under TCG, whose emulator flushes everything on every root switch. The newest
section at the end records the measurements and the open items.
Saved-state testing is an explicit launch choice. The section before it
distinguishes current-program four-level verification from failures in
retained executable snapshots.
The fresh-lab reset is `30572c9f`. Compact recursive desktops landed as `c7571191`,
and the launch request ABI repair as `a287ae39`. No recursive legacy snapshot
upgrade is required. The saved-disk quota/headroom repair remains in place. Current
verification and limitations are recorded in the newest section at the end.
Previous baseline: atomic named creation and deeper recursive matrices at `102e416c`.
The saved-lab launch stages changed program installations in host scratch space, then verifies and
publishes a result that fits the assigned disk extent. Legacy 4 MiB desktop
volumes are backed up and grown offline to 8 MiB without changing their records;
the root's logical allowance is now 2 MiB. Final quota admission follows all
stock and named replacements, with reachability collection under pressure.
Current data and pins survive.
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
python tools/boot-harness-rs/run.py --release --window --fresh
```

This is what the normal [VS Code launch entry](.vscode/launch.json) runs.
**Cathedral: Build & Run in QEMU** uses `--fresh`: it backs up the disk and receipt
under `build/boot-harness-rs/before-fresh-*` and installs current seeds on a fresh
disk on every launch. Recreate users there. **Cathedral: Resume Saved Lab in QEMU**
omits `--fresh` for explicit persistence testing. The owner explicitly does not require compatibility
or recursive migrations for old lab users; use a fresh lab when templates change.
The composition is [distribution/profile.json](source-rs/distribution/profile.json).
It uses x86-64 QEMU/TCG and UEFI.

- The full-screen user picker shows a horizontal row of avatar tiles. **NEW
  USER** copies the hidden template; click a user to enter its workspace. Up
  to three users fit the current bounded listing; the add tile then disappears.
  No scrolling, passwords or logout yet. Restart QEMU to select another user.
- The user's compact bottom taskbar shows **NOTES**, **COLORS**, **SPLIT** and **NESTED** icon tiles. The
  launcher itself and implementation controls are hidden. Click an app tile,
  then the app to focus it. Arrows and Enter work in a focused taskbar.
- **SPLIT** opens two persistent Notes side by side in a separate matrix process.
  Click either child to type. Its own top-right X asks both children about drafts;
  UNSAVED NOTES keeps it open. Save/discard drafts with the child apps' X dialogs.
  F1 saves to separate Left/Right realms; both survive relaunch and cold reboot.
  Outer Settings resizes the two columns while retaining both drafts.
  **DEEPER** checks for unsaved work, then enters another Split with independent
  saved notes. Reopen Split and click DEEPER to return there after a restart.
  Further nesting beyond its funding reports **NO ROOM**, retaining live drafts.
- **NESTED** opens its own user picker and the same desktop, with Notes, Colors
  and another Nested tile. Close other outer apps first. The current budget fits
  four nested desktops beneath the outer one, with Notes at the deepest level.
  Further launches remain bounded by conserved grants. Use Fresh Lab to discard
  stale template snapshots and create users with current code throughout.
  Inner saves survive closing/reopening Nested and reboot. The outer
  taskbar X refuses to close the subtree while an inner app has unsaved work.
- **SETTINGS** at the right of the taskbar (or F2 with taskbar focus) adjusts
  bar height and outer spacing. Save persists and applies the layout to running
  apps; Cancel discards local edits and returns to apps. Apps retain their state.
  SAVED; RESTART means persistence succeeded but live application was refused.
  Retry Save or restart to use the saved arrangement.
- Notes keeps up to eight notes of 240 ASCII characters. Letters, digits, spaces,
  Enter, Left/Right and Backspace edit. **F1** saves/reconciles, **F2** reloads,
  **F3** duplicates the saved note, **F4** switches notes. Colors changes and
  saves its color on **Enter**.
- Notes and Colors have an upper-right **X**. Clean apps exit; dirty Notes offers
  **SAVE / DISCARD / CANCEL**. A failed save stays open. Colors asks **CLOSE
  ANYWAY** if its last storage result is uncertain. Each app-tile click launches
  another instance (up to three, subject to funding). Buttons beside the icons switch
  instances; their X requests close and refuses unsaved work. App-owned X offers
  Save/Discard/Cancel. Multiple Notes instances share saved data but keep separate drafts.
- **END APP** above Settings handles an unresponsive app. It warns about unsaved
  work and defaults to Cancel; confirmation stops the exact observed app/subtree.
  F3 opens it with launcher focus. A recovered launcher can manage the existing
  app. This is delegated authority, not a protected system-wide escape key.
- After storage recovery is exhausted, **RETRY SAVES** appears above Settings.
  Click it (or F4 with strip focus) for one explicit attempt. It preserves live
  drafts and does not reset the automatic restart budget. F1 in Notes first
  reconciles an uncertain save; press again to save an unsaved reconciled draft.
- If the launcher exhausts its retry, **LAUNCHER UNAVAILABLE / TRY AGAIN** takes
  its place. A click or Enter with that screen focused requests one new attempt,
  preserving the app and current arrangement. Failure of that screen stays stopped.
- The default input service negotiates QEMU's absolute virtual mouse. The guest
  cursor follows the host pointer within the window; PS/2 remains the fallback.
  Apps still receive only local matrix input, never the physical device grant.
- The old STORAGE ON/OFF toggle and launcher CLOSE row are gone. Required/
  optional service binding and the matrix LOSS/CLOSE mechanism still exist;
  their engineering controls do not belong in the normal desktop.

Interactive state is stored in **`build/boot-harness-rs/cathedral-storage.raw`**:
the users you made, their notes and colors. Normal F5 sets this disk aside and
starts fresh; **Resume Saved Lab** reuses it. On that saved-lab path, missing media is created; existing
media is never automatically reformatted. The programs are installed into the
image on first launch and republished as new revisions when their bytes
change; the sidecar `cathedral-storage.raw.programs` records what is installed.
The desktop's explicit compatibility upgrade refreshes recognized stock
launchers, Notes/Colors and recognized stock matrix executables in existing
users, preserving their saved data and object identities. Changed installations
save a complete content-named `.before-compact-<digest>.raw` backup before work.
All migration stages run on a private host scratch disk, including reachability
collection under quota pressure. The final result must fit the assigned volume
after any explicit legacy growth and pass cold record comparison before it is
written to the locked disk.
A refused preparation leaves that disk and its installation receipt unchanged.
Existing older migration backups remain useful recovery points. Customized code,
live data, pinned revisions, metadata and saved layout retain their prior rules.
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
chrome, Notes, Colors, Split (with its own Notes code) and a saved `Matrix`
configuration inside it. The user
matrix loads its own launcher path, placement and budgets within the outer
owner's ceiling. `desktop_test.py` drives this from F5
and an independent host model verifies the whole layout; the headless storage
fixture runs a program from its realm, launches a sibling and closes it.

The kernel's contribution stays one syscall for code from sealed objects. It
now allows 48 reader rights per task and a reader page budget of up to twelve
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
  every other instance only forwards and attenuates. No authority policy branches on depth. Stock startup intent may select picker
  or saved-workspace initialization; nothing tunnels through a matrix to use its
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
  runs nominally at 100 Hz, so the 1200-tick base deadline is **twelve
  seconds**, not 1.2 s. Keep units explicit and measure actual spans before
  tuning deadlines.
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
| Kernel | UEFI exit, owned mappings/stacks, frame/heap allocation, interrupts/timer, preemption, isolated user processes, capability syscalls, IPC, admission/teardown from boot templates or sealed objects, recursive resource funding and CPU ceilings; 256 private and up to 512 reader pages per task, sixteen reader rights | [Boot](source-rs/kernel/boot/uefi/main.rs), [user tasks](source-rs/kernel/core/user_tasks/user_tasks.rs), [code grants](source-rs/kernel/core/user_tasks/admission/pool/README.md) |
| Architecture | Shared core with x86-64 and an ARM64 QEMU portability proof; ISA and board responsibilities separated | [Architecture](source-rs/kernel/arch/README.md), [ARM64 bounds](source-rs/kernel/arch/isa/aarch64/README.md) |
| Matrix | Same service recursively owns children, local surfaces, focus, input, realm delegation with scope stamps and attachments, programs run from its realm by owner or child, standing launch policy per slot, close-after-asking, bounded budgets and deadlines; counts unanswered child requests instead of killing a slow child | [Service loop](source-rs/distribution/matrix/service/service.rs), [protocol](source-rs/platform/contracts/matrix.rs), [child launches](source-rs/distribution/matrix/service/children/children.rs), [storage forwarding](source-rs/distribution/matrix/service/parent/storage/storage.rs) |
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

Page budgets cover retained images at each matrix hop and admitted program code.
The root has 640 private/1280 reader pages; the user matrix receives 576/1024.
Chrome receives 96 private pages; each app receives 128/128. Display has 512
reader pages. The kernel ceilings are 1024 private/1536 reader pages with 48
reader rights, and its heap is 512 KiB. Kernel memory objects remain bounded at
512 KiB; the storage content limit remains 256 KiB. Profile funding is explicit.

Recursive storage scopes are folded at each matrix boundary; the original fixed
three-stamp limit no longer bounds tree depth. A shared deadline bounds the exchange.
Collection before every write scans only metadata and the
key-only chunk index. The older 50/100/150-ms collection/commit/save claims were
not reliable unit conversions and are withdrawn. A fresh traced desktop run is
recorded below. The matrix contract's base storage deadline is 1200 **ticks**,
nominally twelve seconds; the runtime's exchange bound is twice that. Values
were not retuned in the recursive-close change. All are lab bounds.

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
- [x] **Remove desktop requirements from shared matrix machinery.** The stock
  executable/configuration/recovery/control router belong to distribution. Platform
  supplies local authority state with caller-selected child capacity and geometry.
  Alternative visual/headless arrangements have host tests without distro imports.
  An independent three-peer executable now exercises real IPC, source authority,
  revocation and replacement with the existing runtime server and shared local
  commands. Runtime child admission is now shared with the stock matrix, including
  channel rollback and retained endpoint custody; the alternative owns its subtree.
  Upstream/rendering orchestration still needs a second consumer before extraction.
- [x] **Give ordinary apps scoped service acquisition.** One broker connection
  at startup replaces their positional interaction/storage bundle. The sealed
  manifest bounds the offer; endpoint/incarnation selects caller authority.
  Notes and Colors use it in F5. Repeat requests allocate nothing; interrupted
  handoffs reclaim retained rights. Optional/undeclared/incompatible imports and
  cached bindings have host, real-IPC and disposable desktop coverage. This does
  not yet start providers lazily or grant new services after launch.
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

The matrix's [loss survey](source-rs/distribution/matrix/service/children/loss.rs)
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

[Configuration](source-rs/distribution/matrix/contracts/config.rs) is the stock
matrix's bounded 104-byte `CTHMAT01` record: launcher path, page/reader budgets,
margin, launcher width, gap, app surfaces and key mask. The
[loader](source-rs/distribution/matrix/service/children/configuration.rs)
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
focus. It edits strip width (192ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã¢â‚¬Å“320 pixels) and outer spacing (8ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã¢â‚¬Å“40 pixels).
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
Start at [configuration.rs](source-rs/distribution/matrix/service/children/configuration.rs)
for the grant and standing-placement update, then
[layout.rs](source-rs/platform/libraries/matrix/world/layout.rs) for authority and
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
the observer. Start at [children.rs](source-rs/distribution/matrix/service/children/children.rs)
for admission/placement, [lifecycle.rs](source-rs/distribution/matrix/service/children/lifecycle.rs)
for outcome/cleanup, and [launches.rs](source-rs/platform/libraries/matrix/world/launches.rs)
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

Start at [recovery.rs](source-rs/distribution/matrix/service/children/recovery/recovery.rs).
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

## Stock matrix and reusable platform boundary, 2026-10-18

The stock matrix is now [a distribution executable](source-rs/distribution/matrix/main.rs),
`cathedral-stock-matrix`. Its launcher/app roles, two-child owner router, sibling
launch/close requests, saved strip arrangement and configured-launcher recovery
are optional stock policy. Their [extension contract](source-rs/distribution/matrix/contracts/matrix.rs)
and unchanged configuration codec live beside that program. The normal profile,
focused profiles, installer and fault builds use the new package. Wire operation
numbers and saved configuration bytes are unchanged; there is no data migration.

[The platform library](source-rs/platform/libraries/matrix/README.md), `cathedral-matrix`,
retains reusable authority and geometry. `World<N>`, storage scopes, launch
observations and management ledgers take independently selected child capacities.
Layout accepts owner-supplied rectangles; no slot is intrinsically a launcher.
Surface ranges may be explicitly selected or allocated from free space, cannot
overlap, and cannot move while live. Pointer hit testing follows those ranges.
Replacement snapshots retain their original range without changing siblings.
The stock executable explicitly chooses its previous bottom/top sibling ordering.

This is not yet a general-purpose matrix server runtime: the existing IPC loop,
admission orchestration and forwarding adapters remain in the stock executable.
Another distribution can use the platform mechanisms and contracts with its own
program; it does not yet get a separately packaged configurable server loop.
The new host proofs exercise three visual children (last slot controlling layout),
arbitrary owner stacking, four-child headless ledgers, and zero visual slots.
These prove removal of the two-slot/desktop assumptions from shared state, not
that a phone or embedded distribution has been built. Fixed protocol resource
bounds remain. Kernel implementation and ABI are unchanged.

Architecture checks now reject platform/kernel imports of the stock contract,
and reject implementation dependencies from that contract. Reading paths and
package maps were updated across executable/IPC seams; `mod.rs` stays plumbing.

Verification and findings:

- Workspace host tests passed, including 30 shared matrix tests and the moved
  configuration codec test. Userspace clippy and stock matrix fault/trace feature
  clippy passed, as did the shell/live-probe feature set and restart-budget tests.
- `desktop/20261019T011839687704Z` passed the default F5 flow and cold persistence;
  the independent oracle checked all 235 records. This preceded the explicit
  stacking correction described below.
- `launcher-recovery/20261019T012031252420Z` passed recover, startup and
  owner-revocation cases, including live layout, automatic and explicit retries,
  retained app draft, save/close and cold persistence. Source media was unchanged.
- The recursive desktop test caught an introduced stacking regression: first-fit
  range allocation made launch order determine overlap order. The fix makes the
  owner's stacking choice explicit, tests arbitrary order and replacement on the
  host, and keeps the stock bottom/top policy in the distribution. The recursive
  desktop then passed nested pixels, focus, lifecycle recovery and cold colors.
- The management fixture initially exhausted the 4 MiB engine before boot while
  upgrading old code and installing two probes. Its preparation now compacts its
  disposable copy before each injection; it never changes source/interactive media.
  Compaction alone was insufficient: the host injector rewrote every unchanged
  record too. It now inserts only changed records, preserving unrelated pages.
- `management/20261019T013555980484Z` then passed all three scenarios on the
  final code: hung app termination, both Cancel actions, stale/ungranted authority,
  recovered launcher management, replacement apps and cold persistence. The
  source fixture was unchanged. Its successful setup also exercises the corrected
  host injector on an upgraded image, with both fault programs installed.
- Two recursive storage runs reached an ATA FLUSH timeout (`op=e7`, status `c0`,
  50 ticks) and the provider's existing engine-failure panic. The second ran after
  host tests finished. This remains the tracked ATA completion issue, not a pass;
  no storage driver/deadline changes are part of this boundary cleanup. Logs are
  `build/boot-harness-rs/matrix-storage-timeout-20261019.log` and
  `build/boot-harness-rs/matrix-storage-test/serial.log`.
- The later default-desktop run `desktop/20261019T013226419025Z` hit the existing
  ATA WRITE completion timeout before login (`op=30`, LBA 24, status `d0`, 50
  ticks). The stock recovery screen appeared; this is not a passing desktop run.
- Layout checks passed (35 Omega/106 Rust package edges), including six checker
  unit tests. Named-body formatting, changed Python compilation, Markdown links
  and diff checks passed. The host installer checks with its locked engine feature
  set. ARM64 was not rerun: this changes no kernel/architecture implementation.

Follow-up architecture proof: compose a small independently owned headless or
alternative-layout matrix program, then extract the shared IPC orchestration it
actually needs. Do not move stock config/recovery policy back into platform or
invent a universal desktop framework. The outstanding delegated live-storage
retry and protected-chrome owner question remain separate work.

## Independent matrix over real IPC, 2026-10-18

The [alternative matrix](source-rs/distribution/lab/matrix-composition/matrix/main.rs)
is a separately compiled program with three equal virtual input regions. It has
no stock matrix, desktop contract, saved configuration, storage or device grant.
It composes `World<3>` with the existing runtime `Server`; the latter already
handles more than two peers and orders disconnection before reconnection.
The [proof](source-rs/distribution/lab/matrix-composition/README.md) admits this
program and three isolated peer processes through explicit links. It checks
local-only views, malformed packets, denied owner/source requests, input to the
third region, revocation, and replacement requiring a fresh grant while siblings
keep their state. The first bring-up used the supervisor role for the relays;
setting their explicit profile argument corrected that fixture wiring mistake.

[Local command dispatch](source-rs/platform/libraries/matrix/world/commands.rs)
now owns common placement/revocation and trusted-source input. Both programs use
it; F5 also uses it on the physical input connection. Endpoint authentication,
parent binding, stacking, configuration and launch policy stay with each program.
Children enter the separate restricted dispatcher. No protocol numbers, persisted
bytes, device deadlines or kernel implementation changed.

This closes the lack of a real independent IPC consumer for the local-world API.
It does not yet extract the stock admission, upstream, storage or rendering loops:
the test supervisor admits these peers as siblings. There is no need to invent
another generic server abstraction for the subset that the runtime already serves.
The source-layout check now rejects all distribution dependencies from this proof's
packages, so adding a stock contract cannot quietly invalidate the evidence.

Verification:

- `python tools/boot-harness-rs/matrix_composition_test.py` passed in
  `matrix-composition/20261019T021047276260Z`, using private firmware/ESP and no disk.
- `run.py --matrix-test --release --timeout 120` passed the stock recursive
  matrix's locality, input, teardown and recovery checks after shared dispatch.
- `desktop/20261019T021239109327Z` passed the default F5 user/app flow and cold
  persistence. The independent host oracle verified all 235 records. This run
  had no ATA timeout; it does not resolve the previously recorded occurrences.
- Workspace host tests and `cargo check-userspace` passed. Layout checks passed
  (35 Omega/113 Rust edges) with seven checker tests; named-body formatting and
  diff checks passed.

The prior ATA completion failures and live-storage retry gap remain unresolved.
The next matrix extraction should target independent child admission or upstream
adaptation, with both actual consumers preserving their distinct policy.

## Shared child admission and independent subtree, 2026-10-18

[Runtime admission](source-rs/platform/libraries/user-runtime/matrix/admission/README.md)
now owns channel construction, startup-endpoint transfer, child spawn and control
connection acceptance. It closes all endpoints on pre-spawn failure, or cancels
and collects the child before closing retained parent ends on control-connect
failure. Successful admission returns the task and retained connections; callers
choose installation, authority grants, teardown and reservation release. There is
no implicit Drop cancellation. The stock matrix uses it for template, catalogue
and stored-object admission, preserving its budgets and recovery policy. The
runtime matrix folder has a named entrance and plumbing-only `mod.rs` files.

The independent matrix now receives three launch loans and admits its own
children. Its supervisor has neither their launch authority nor their control
ports while the matrix is alive. Children receive the normal three-endpoint
matrix environment, no storage pair, no static peer links, and no launch/device
authority. The matrix's small asynchronous lab probe loop keeps serving children
while it waits for their control replies. Probe failure ends this test program;
that deliberately limited behavior is not a shared application recovery policy.

The QEMU proof checks, per matrix startup, sixteen busy-spawn failures after
channel construction and sixteen channel-budget failures after allocating two of
three required channels. Every failure preserves the original resource charges.
It also checks sixteen middle-child replacements, no inherited local grant,
unaffected sibling views, whole-subtree cancellation and a fresh matrix using the
returned launch loans. These prove actual custody/rollback, not just pure state.
The control-connect failure branch was preserved but not separately injected.

Verification:

- `matrix-composition/20261019T022726897765Z` passed owned admission, both rollback
  paths, local input, denied authority, replacement and subtree reclamation.
- `run.py --matrix-test --release --timeout 120` passed stock recursive admission,
  locality, input, teardown and recovery with the shared helper.
- `desktop/20261019T022923861143Z` passed the default F5 desktop and cold-reboot
  persistence; its host oracle verified all 237 records. No ATA timeout occurred
  in this run; the previously recorded failures remain unresolved.
- Workspace host tests, `cargo check-userspace`, layout (35 Omega/113 Rust edges),
  seven layout-checker tests, named-body formatting and diff checks passed.

No kernel ABI, stored schema, channel budget or device deadline changed for the
ordinary desktop. The independent profile funds eleven channels explicitly; this
leaves two spare with three children live and makes partial-allocation rollback
observable. Remaining matrix extraction targets are upstream adaptation and
rendering/storage forwarding; alternative dynamic-reservation policy is not yet
proved. The recorded ATA issue and live-storage retry gap remain separate work.

## Scoped application bootstrap, 2026-10-18

The owner-approved [bootstrap model](source-rs/platform/contracts/services/README.md)
separates manifest requests, launch-approved authority, and acquired connections.
Applications enter with `Services` and request interaction/storage before their
ordinary loops. The platform owns the protocol, client, endpoint custody and
admission mechanics; the stock matrix authenticates its child connection and
selects its approved offer. No PID, sibling number or requested realm in a packet
can choose another offer. There is no new kernel syscall or global service locator.

`CTHIMP02` opts sealed stored programs into broker startup. Older import records
and undeclared legacy code keep their existing bundles. The runtime compatibility
path also supports explicitly wired catalogue fixtures. The new environment has
two endpoints for one logical broker; existing kernel owner control is separate.
The current interaction service shares the broker RPC pair and acquires its wake
receiver. Storage acquires a separate pair in two endpoint-transfer replies.
Service and bootstrap versions are separate. Required imports still refuse launch
when missing/incompatible; unavailable optional ones remain absent.

Channels are allocated before spawn from existing budgets. Acquisition only moves
preapproved rights, each at most once. Repeat application calls cache bindings;
repeat wire requests cannot duplicate them. Unsent rights remain owned by the
broker and are closed on task teardown, including interrupted pair acquisition.
Ambiguous client transport failure closes the shared broker/interaction stream so
a late capability reply cannot contaminate a subsequent ordinary RPC. Acquisition
must be serialized with interaction; current apps acquire during startup.

The first F5 run (`desktop/20261019T033724803621Z`) found a real ordering bug:
notification preceded wake-receiver transfer, so the kernel refused the nonempty
queue. The stock owner now preserves pending notification state until that receiver
has transferred. Expected transfer capacity/busy errors return refusals rather
than ending the matrix. The later default desktop passed.

Verification:

- `matrix-composition/20261019T034140088220Z` passed acquisition, unknown/ungranted
  and version refusals, forged identity fields, repeated raw/cache requests,
  rollback, replacement and whole-subtree reclamation. Sixteen cycles interrupt
  acquisition before any service or after one storage endpoint. The third peer's
  storage endpoints test custody only; this headless fixture has no filesystem.
  Its thirteen channels fund eleven live channels plus two for partial rollback.
- `desktop/20261019T033908741405Z` passed ordinary F5 Notes/Colors behavior and cold
  persistence; the independent host oracle verified 239 records.
- `desktop_test.py --service-imports undeclared` passed in
  `desktop/20261019T034306507571Z`: a disposable Notes declaration omits storage,
  which stays absent even though the owner offers it. Editing, dirty-close and
  independent Colors persistence pass across cold reboot.
- `desktop_test.py --service-imports incompatible` passed in
  `desktop/20261019T034431375347Z` with optional storage version 2. Notes degrades
  to temporary storage; Colors retains its separate usable version-1 binding.
  These variants use visual/serial checks; the default test alone runs the full
  independent retained-record oracle. Interactive saved media was untouched.
- `run.py --matrix-test --release --timeout 120` passed legacy recursive matrix
  startup/locality/recovery. `--matrix-storage-test` passed existing scoped roots,
  recursive delegation, revocation, realm launches, siblings and replacement.
- Workspace host tests and `cargo check-userspace` passed. Notes clippy also passed
  startup-fault/admitted-fault/management/render/trace feature composition.
  Layout passed (35 Omega/113 Rust edges), with seven checker tests, and named-body
  formatting passed.

This is a bounded bootstrap, not arbitrary service enumeration, dynamic grant
addition or lazy provider startup. All service channels remain preallocated.
The stock broker path currently requires interaction and separate storage streams;
legacy headless multiplexing retains its prior protocol. Upstream/rendering
extraction, the recorded ATA completion issue and live-session storage retry
remain separate work. No disk schema, kernel ABI or default desktop budget changed.

## Visible alternative matrix, 2026-10-18

[Split](source-rs/distribution/applications/split/README.md) is installed under the
stock template and recognized existing users, with its Notes code beneath it.
The normal executable-only chooser discovers it by path/metadata. No demo-specific
launcher dispatch or boot catalogue grant is involved. It enters through Services,
uses its own storage binding to load code, validates sealed imports, and admits two
independent Notes processes with interaction-only offers. Both apps prove storage
is absent and cannot be reached through interaction. Their temporary drafts never
enter the object store.

The shared mechanisms now include `World::inherit_poll`, synchronous upstream
exchanges and retained image custody, alongside the existing admission, service
broker, geometry, surface naming and input rules. The stock parent consumes the
same snapshot/custody code, including its asynchronous presentation commits.
Split imports no stock implementation or desktop contracts; the layout checker
rejects such dependencies. Its named session entrance leads to children, requests,
rendering and loss leaves; mod.rs only wires them.

The enclosing matrix controls Split's placement; Split computes its children's
two columns from its local extent. It shares sealed image readers, never maps or
copies descendant pixels. An asynchronous LOSS survey continues serving children,
and silence/malformed replies prevent close. After a timed-out child control
exchange that stream stays uncertain until child death. Individual closes retire
surfaces before returning reservations; whole-program exit lets the kernel reclaim
its subtree. The visible proof repeatedly starts the same program after teardown.

### Lab funding and limits

The larger live subtree exposed several old ceilings. Funding is explicit:

- Root: 256 private pages, 384 reader pages, 28 channels and six dynamic slots.
  Display/input reserve no unused dynamic channels. Initial owner plus root still
  reserve only the existing 32-channel global pool.
- User: 160 private pages, 192 readers, 16 channels and four descendant slots.
  Its launcher receives 64 private pages; its app receives 80 private/64 reader
  pages, six channels and two descendant slots. This is stock distribution policy.
- Split lends each Notes 32 private/four reader pages, no channels or descendants.
  It retains bounded header images within its remaining sixteen private pages.
- Single objects remain capped at 128 pages; the separate private-budget ceiling
  is now 256. Per-task reader slots are sixteen, with an independently bounded
  global copied-reader pool of 128 (the IPC retirement bitmap remains u128).
- Compositor surface capacity remains sixteen. Login overlaps user admission with
  five picker/eleven user surfaces; after stopping the picker, the owner grants
  fourteen to the user. The launcher uses five, Split nine (four per temporary
  Notes plus its header). Temporary Notes no longer allocates two unused surfaces.

Named additions allocate fresh object IDs; original six-object bootstrap identities
remain valid. The installer receipt authorizes exact previous addition manifests,
not arbitrary programs with matching names. Custom entries block installation into
that path. Migration preserves saved content, identities, launcher choice and
geometry; only known old resource defaults change. It has a separate
`.before-realm-programs.raw` backup and unchanged installations do not reopen the
image. The independent desktop oracle now models both added programs and their
nested subtree copy.

### Remaining boundaries

Split is a bounded alternative implementation, not a general replacement desktop.
It does not forward child storage, accept presentation transactions, dynamically
choose children or restart them. Parent snapshot handling and bitmap custody are
shared; full transaction scheduling and root-display adaptation remain in the stock
matrix. Loss of upstream transport ends this subtree rather than recovering it.
Parent revocation/regrant is covered by the pure World test; live outer resize and
whole-subtree close are covered visually. No owner semantic decision is needed for
these implementation gaps.

Next useful work: finish extracting transactional presentation and scoped storage
forwarding when a second consumer needs them; diagnose the recorded ATA WRITE/FLUSH
completion timeout; then address live-session storage retry delegation. Keep the
F5 desktop as the ordinary way to use new capabilities. No authenticated login,
Warden key handling or encrypted realm unlock is implemented by this milestone.

### Verification

- Workspace `cargo test --locked`, `cargo check-userspace` and `cargo check-uefi`
  passed. After adding the final revocation/budget regressions, core's 108 and
  matrix's 31 tests passed again. Storage host tests passed all 34 cases, including
  named installation, exact receipt updates and preservation of custom programs.
- Harness Python tests (13), layout tests (7), layout check (35 Omega/118 Rust
  edges), named-body formatting and changed Markdown links passed.
- `desktop/20261019T064749690692Z`: ordinary F5 Notes/Colors and cold persistence;
  the independent host oracle verified all 300 records.
- `desktop/20261019T064900399844Z`: `desktop_test.py --nested` passed two cold boots,
  independent child focus/drafts, denied persistence, dirty-close, outer resize
  and three complete subtree launches/teardowns per boot.
- `labels-upgrade/20261019T065815917067Z`: `labels_upgrade_test.py --nested` copied
  the retained six-program desktop disk, preserved both saved notes and identities,
  installed Split, and passed the same nested interaction/reclamation checks inside
  the migrated user. An unchanged second installation left the disk byte-identical.
- x86 release smoke, ARM64 release portability, stock `--matrix-test` and
  `--matrix-storage-test` passed. Independent matrix composition passed in
  `matrix-composition/20261019T065431646805Z`, including interrupted acquisition,
  rollback and subtree reclamation.

All disk tests used disposable images or copies. The interactive saved disk was
not opened or modified. The previously recorded intermittent ATA completion issue
and live-session storage retry gap remain outstanding; these passing runs do not
claim to resolve them.


## F5 installation capacity repair, 2026-10-19

The actual saved desktop failed before QEMU started. In-place redb compaction
requested 4,296,704 bytes inside the 4,186,112-byte engine extent. Rebuilding alone
was insufficient: subsequent seeding also needed temporary allocation, and adding
Split crossed the logical quota while old unpinned history remained.

The host [install entrance](tools/storage-lab/engines/volume/install.rs) now
stages seed, stock-copy refresh and hierarchy/named additions together on a private
host scratch volume, with the original image locked. A content-named full backup
is synced first. Rebuilding uses separate scratch capacity, then enforces the
original extent and cold-compares every prepared record before publishing. A failed
preparation leaves the original disk and sidecar unchanged. Named additions now
collect unreachable history under quota pressure after staging their new heads;
current data and pins survive. Actual live/pinned excess still refuses.

There is no kernel, guest-volume-size or syscall change. This remains offline
image maintenance: publication errors attempt restoration, but process/power loss
during the final copy requires restoring the synced backup. Individual migration
backups printed inside the temporary directory are not durable recovery points;
the `.before-compact-<digest>.raw` beside the saved disk is the recovery point.

Verification:

- Storage host tests: 36 passed; storage host clippy passed with warnings denied.
  The new near-capacity fixture demonstrates that
  the old in-place compactor fails, the rebuild preserves all records, and failed
  edits/oversized results leave the original bytes unchanged. A named-addition
  quota test preserves current user content while collecting old revisions.
- Harness Python tests: 13 passed. Changed installation invokes one staged host
  operation; unchanged installation never opens the image; failed preparation
  never records success.
- `desktop/20261019T071938153217Z`: ordinary Notes/Colors interaction and cold
  persistence passed; independent host oracle verified all 300 records.
- `launch-repair/fixed-copy`: a copy of the exact failing saved disk completed
  installation and booted to the user picker (3.16 seconds after QMP connection).
  The normal `--release --window` command then upgraded the actual saved disk and
  opened QEMU. It retains the same extent; its full original backup is
  `cathedral-storage.raw.before-compact-872824a45ab20d65f4d9dfd3a1085b2523044e6c740aba827a93159319575b6c.raw`.
- `labels-upgrade/20261019T072105675488Z`: existing-user saved-notes compatibility
  passed. Its subsequent `--nested` pass failed at live layout Save: the provider
  stalled and was collected, leaving SAVE FAILED and the old placement. This is
  not a passing nested regression. The trace includes ATA FLUSH completion work;
  the precise cause was not diagnosed here. The recorded runtime storage/retry
  issues remain open, separately from this pre-QEMU installation fix.


## Storage growth diagnosis and bounded PIO (2026-10-19)

The retained-disk nested Split layout-save stall is reproducible and diagnosed.
Trace `labels-upgrade/20261019T170130555475Z` enters extent growth from
1,503,232 to 3,002,368 bytes at tick 1525 and the root retires its stream at
tick 1962 (600 ticks after the request). Zeroing the new extent through
single-sector PIO commands exceeds the request deadline. This evidence does
not explain the older intermittent ATA busy timeout.

The block-device boundary now admits contiguous sector slices, with checked
fallbacks. BlockRegion batches aligned interiors while preserving partial-sector
read/modify/write and whole-range confinement. ATA uses at most 32 sectors per
command; kernel register validation admits exactly that bounded count, and each
kernel data transfer still checks readiness and transfers only 512 bytes. No DMA,
write retry, enlarged deadline or flush reordering was introduced. Engine growth
zeros through an immutable 16 KiB buffer in read-only image memory. A development
attempt using stack storage killed the provider during commit; moving the buffer
off its 32 KiB stack fixed that regression.

`labels_upgrade_test.py --nested --trace-storage` now passes on the same retained
source disk: `labels-upgrade/20261019T170820481411Z`. Growth takes 111 ticks,
commit 122 ticks, and both resized drafts, three subtree launches/teardowns and
original saved notes pass. Ordinary desktop with tracing also passes cold reboot
and the exact 302-record host oracle: `desktop/20261019T171005498588Z`.
The interactive saved disk was not used or changed. Full default host tests,
engine/volume failure-injection tests, both default target clippy aliases, format
and layout checks pass. New host checks cover contiguous range rejection before
IO, actual aligned batch dispatch, and retiring after an uncertain batch write.

Tracing now records extent begin/end and stream retirement with deadline and
error; timing summaries write retirements separately. ATA read/write counters
count requested sectors; flushes remain command counts.

Still in progress: explicit storage retry after automatic recovery exhaustion;
shared asynchronous scoped forwarding and persistent isolated Split children;
resource/admission diagnostics for that nested composition. Split still ships
with temporary children at this checkpoint.


## Nested persistence and explicit save recovery (2026-10-19)

The ATA growth fix above is pushed as **`581d6175`**. It was rebased over the
independent Omega UEFI baseline update `e298c25a`; those changes are preserved.
This follow-up makes the same F5 desktop exercise persistent nested applications.

### Current behavior and boundaries

- Split's two Notes instances receive separate storage channels rooted at named
  `Left`/`Right` containers beneath Split. Saved text survives subtree teardown
  and cold reboot. Split still has no stock-matrix contract or implementation
  dependency. Its named [storage entrance](source-rs/distribution/applications/split/session/storage/storage.rs)
  sequences refresh, authorization and asynchronous forwarding; realm creation
  lives in its sibling `realms.rs`.
- Platform `Scopes::prepare` now supplies connection/revision validation and
  append-only scope stamping to both implementations. The shared runtime
  `matrix/upstream/exchange.rs` polls one bounded packet/attachment reply without
  blocking interaction. A changed upstream binding invalidates child revisions;
  Split restores only its standing child grants within the same newly authorized
  realm. Transport uncertainty retires the stream rather than reusing late replies.
- Notes uses four surfaces regardless of storage availability; its tab bar is one
  small bitmap. Split retains nine surfaces total, so the surface pool stays 16.
- The stock launcher displays **RETRY SAVES** when the owner reports recovery
  exhaustion/failure. Click it, or F4 with strip focus, for one explicit attempt.
  The action passes through grants bound to live interaction connections (launcher
  to user matrix to root); ordinary Notes is denied both retry and availability.
  Only the desktop owner replaces the provider. Queued attempts are deduplicated,
  revoked on grant changes, and checked again against the live direct child before
  consumption. Success/failure never replenishes automatic restart authority.
  Live apps and drafts remain. The root dispatcher and its small authority model
  are in `distribution/matrix/service/parent/recovery/`; provider ownership stays
  in `distribution/desktop/session/persistence/`.
- The IPC backing pool is 36 channels; an account can hold 32 channel reservations
  and 24 dynamic handles. F5 funds root/user/app channel ceilings at 32/20/10;
  ordinary leaf apps still create none. Two persistent child admissions need ten
  channels and up to 23 handles while their service offers remain pending. These
  are explicit bounds, not global resource discovery. Admission failures now log
  the failed reserve/start stage, requested funding and local limits/charges.
  The initial prototype forgot the desktop's runtime funding override and then
  hit the old handle cap; both are corrected and recorded by the diagnostics.

### Evidence

- Default host tests and 13 harness unit tests pass. Recovery policy tests cover
  consumed restart authority plus stale, revoked, replaced, duplicate and dead
  requesters. Shared scope tests cover stale revision, connection substitution
  and refusing a full stamp chain without dropping an ancestor scope.
- Default userspace/UEFI clippy and the changed shell/live-recovery/trace,
  Notes management, chooser recovery and storage recovery feature set pass.
  Layout still enforces 35 Omega/118 Rust edges; no dependency edges were added.
- `desktop_test.py --nested --trace-storage`:
  `desktop/20261019T172240376279Z` passes separate saves, resize with two drafts,
  dirty close, three complete admissions/teardowns and a cold reboot.
- `provider_live_test.py` on a copied ordinary desktop:
  `provider-live/20261019T172352620786Z` passes precommit and postcommit loss,
  automatic restart exhaustion followed by visible explicit retry and durable
  save of the retained draft, and a failed explicit readiness retry that remains
  bounded while preserving the draft. Every case cold-opens the expected data.
- New `provider_live_test.py --case nested` on a copied Split desktop:
  `provider-live/20261019T172735251575Z` passes failure/replacement/regrant through
  root/user/Split, preserves the live child draft and unchanged sibling bitmap,
  reconciles, saves and cold-opens the result. Both Notes tasks remain alive during
  recovery. The provider probe feature only changes where it observes a dirty
  draft; production supervision and forwarding handle the failure.
- Normal `desktop_test.py --trace-storage`:
  `desktop/20261019T172830077591Z` passes the full F5 UI, cold reboot and exact
  308-record host oracle. `--matrix-storage-test` passes scoped roots, recursive
  delegation, revocation, copies, realm-loaded programs and provider replacement.
- x86-64 smoke, dynamic IPC, admission suites and the ARM64 portability proof
  pass after the changed channel/handle ceilings.
- Retained-disk upgrade plus nested saves passes on the original older fixture:
  `labels-upgrade/20261019T173130306794Z`. Installer idempotence and both existing
  saved notes pass, followed by separate Split saves and complete teardown.

### Remaining limits and sensible follow-up

The old intermittent ATA busy timeout is still not explained by the diagnosed
large extent-growth stall. Keep the traces; do not claim that all stalls are fixed.
The current protocol admits three nonzero scope stamps. Root/user/Split forwarding
uses 600/700/800-tick bounds under the client's 1200-tick ceiling; arbitrary depth
needs a protocol-level treatment instead of repeating the stock one-hop constant.

First-time creation and naming of Left/Right use separate durable transactions.
An interruption between them can leave an unnamed empty container in Split's own
realm. Existing named data is never overwritten; atomic named creation is the
next useful storage operation to design and test. Split has no child restart UI,
dynamic child chooser or presentation batches yet. A permanently failed parent
transport still requires subtree relaunch; ordinary provider replacement through
an intact parent connection is covered above. No new owner semantic decision
was needed for this milestone, and the interactive saved disk was not altered.

## Atomic creation and another real matrix level (2026-10-19)

This section supersedes the previous checkpoint's separate create/name operation,
three-stamp depth cap, per-depth deadlines and resource ceilings.

### Current behavior and reading path

- F5 -> user -> Split -> **DEEPER** now runs the same Split executable beneath
  another saved realm, with independent Notes children. It checks both current
  children's loss reports before replacing them. Reopening Split returns to its
  normal two children; DEEPER revisits the saved nested workspace. A further
  unfunded level says **NO ROOM** without discarding either draft.
- `CREATE_NAMED` commits initial contents, hierarchy metadata, name/type indexes,
  revision/event and quota together. Split uses it for Left/Right. An existing
  name returns a conflict without changing anything; uncertain callers reconcile
  by lookup/head. The sealed request includes 64 metadata bytes within the
  existing 256 KiB payload ceiling. Later publication still permits 256 KiB.
- Named shallow `COPY` shares the source's content and atomically installs its
  new hierarchy name. DEEPER uses it for its executable, then named subtree copies
  for Notes/Left/Right. `COPY_TREE` still refuses copying a subtree into itself.
  Setup is several individually atomic operations, repaired by name on retry;
  the new child is not admitted until preparation finishes. Existing nested
  contents are never overwritten.
- The shared platform scope policy uses two packet scope slots and a read-only
  containment proof when another restriction will not fit. Only successful proof
  plus renewed caller-revision validation permits the folded original request.
  Attachments remain held locally during proof. This works because hierarchy
  parents are immutable and object IDs are never reused; moves or aliases would
  require revisiting it. Tests simulate eight levels and reject escaped scopes.
- Word seven carries one absolute deadline through refresh, containment checks
  and the actual request. Each hop may shorten it, never extend it. Expired
  requests are refused before touching upstream IPC; the next request remains
  usable. The root retains its 600-tick provider bound under the client's
  1200-tick exchange bound. Active durable commits can still have uncertain
  outcomes when a response is lost.
- Start at Split's `session/session.rs`, then `children/children.rs` for admission
  and `children/nesting.rs` for the deeper transition. `children/image.rs` holds
  executable reading/validation. Its `mod.rs` is only Rust plumbing. Platform
  policy remains in `matrix/world/storage/storage.rs`; stock and Split transport
  orchestration remain in their own service/session implementations.

### Funding and kernel correction

The global dynamic channel pool is 40, with up to 36 reservations and the existing
24 dynamic handles per account. F5 root/user/app channel funding is 36/25/15;
private pages are 256/208/128 and reader pages 384/256/128. Root funds seven dynamic
slots, user five and app three. Inner Split receives 80 private pages, 64 reader
pages, ten channels and two descendants. App presentation funding is ten surfaces;
inner Split uses nine and the global surface pool remains sixteen. Picker-to-user
overlap is included. These are explicit admission bounds, not promised unlimited
recursion. The installer upgrades only recognized stock funding configurations.

Reusing a collected task slot under a different parent exposed a kernel control
pair assertion that assumed the old owner would return. `ipc/bootstrap.rs` now
requires the old child's side closed and a fresh epoch, then clears old grants,
queued messages and escrow before installing the new authorized pair. Live service
pair replacement keeps its stricter same-peer checks. A regression proves the old
parent cannot use its handles and the new parent cannot receive an old reply.

### Verification

- Workspace host tests, 14 database-object host tests, 36 storage-lab engine
  tests and 13 harness tests pass. Named creation tests include collision/invalid
  inputs, transaction abort and cold opening a snapshot after every backend write:
  each image exposes either the old state or the complete named object and indexes.
- Default userspace/UEFI and changed deep-recovery/trace clippy checks pass.
  Layout still enforces 35 Omega/118 Rust edges; no implementation dependency
  between Split and the stock matrix was introduced.
- `desktop_test.py --deep --trace-storage` passes repeated admission/collection,
  independent durable edits, dirty close, resource refusal and cold reboot:
  `desktop/20261019T185329329096Z`.
- The complete live provider suite passes precommit/postcommit loss, exhausted
  automatic recovery and explicit retry, readiness-stall refusal, nested and
  deeper replacement: `provider-live/20261019T185352551039Z`. Drafts and the
  unchanged sibling survive; live children remain running, regain storage and
  save durably across cold reboot, including root/user/Split/Split/Notes.
- `--matrix-storage-test --release` passes scoped delegation/revocation, copies,
  realm-loaded programs and replacement, plus full-packet folding, denied foreign
  scopes and an expired request followed by a successful fresh request.
- Normal desktop plus exact 312-record host oracle passes:
  `desktop/20261019T185007998065Z`. Retained-disk upgrade, installer idempotence,
  preservation of existing notes and independent Split saves pass:
  `labels-upgrade/20261019T185217790387Z`.
- x86-64 smoke, dynamic IPC and admission suites and the ARM64 portability proof
  pass after the kernel reparenting correction and funding changes.

### Remaining limits and follow-up

Containment proofs add reads as depth grows; no proof cache or arbitrary-depth
performance claim is made. Nested executables/data are snapshots, not managed
program updates. The experimental wire changed, so arbitrary old compiled nested
snapshots are not promised compatibility. Whole workspace preparation is not one
transaction, though each named operation is atomic and retryable.

Split still lacks a child restart UI, dynamic chooser and presentation batches.
A permanently failed parent transport requires subtree relaunch. Provider
replacement through an intact parent link restores standing grants without
restarting live children. The old intermittent ATA busy timeout remains unexplained.
No new owner decision was needed. Tests used disposable/copied disks; the
interactive saved disk was not altered.

## 2026-10-19 ? Multiple instances and replaceable visual roles

F5's stock matrix now has five local child slots: chrome, three app instances and
background. No kernel or platform code knows these desktop roles. The background
is resolved at `Background`; chrome still comes from the persisted launcher path.
Custom executable bytes remain protected by the existing installer rules. The
background has one bottom surface, no keys/storage/descendants/channel creation.
It sees pointer events only where no higher child is hit. There is no role-selection
UI or automatic background restart yet.

Start at `distribution/matrix/service/children/children.rs` for owner dispatch;
`configuration.rs` establishes roles; `launch.rs` admits scoped code; `workspace.rs`
selects and lists local instances; `lifecycle.rs` handles close/death. Chrome's
`session/tasks.rs` paints running-app rows. Its instance tickets name only this
matrix's live apps. Ordinary Notes is checked to receive DENIED for enumeration
and selection. The platform's new owner-only visibility operation retains geometry
and local scenes, cancels input, and clips hidden publications to zero upstream.
It does not suspend processes or give them information about siblings.

Resource changes: 15 kernel task accounts, 96 dynamic channels (92 reservable),
32-handle waits, 48 reader rights per task, 48 dynamic handles and 512 KiB kernel heaps on both ISAs. Per-task
private/reader budget ceilings are 1024/1536 pages; individual objects remain 128
pages. The stock scene has 40 surface slots. These are bounded lab capacities,
not a scalable production allocator. Runtime wait serialization now sizes its
handle array from the contract; the old 64-byte message buffer only held eight.
`CTHIMP03` adds channel/descendant request ceilings to scoped service declarations;
leaf apps request zero, Split requests its subtree funding. Owner offers still cap
actual grants. Older declarations preserve their former funding behavior.

The compositor now builds immutable image views once per redraw, rather than
once per 256-pixel scanline fragment. The desktop test measures QMP key injection
to screenshot-observed text (including capture/polling overhead); this is an upper
bound, not a renderer CPU benchmark. It does not establish a frame-rate guarantee.

Outstanding: full nested desktop/login, arbitrary dynamic app counts, richer
swappable background grants, background recovery, keyboard task selection and
further measured chrome/compositor work. Selection currently replays surfaces
individually rather than publishing a whole switch atomically. Multiple instances
share their executable's realm; no implicit per-instance copy is made. Split's
subtree can hit funding limits before all three app slots are occupied. Existing
storage revision/conflict semantics remain in force.

Verification passed:

- `cargo test --locked` in `source-rs/`; storage-lab host tests with `--features engines`.
- `cargo check-userspace`, `cargo check-uefi`, source-layout (35 Omega / 121 Rust edges),
  Rust formatting and the 13 discovered harness unit tests.
- Normal `desktop_test.py --trace-storage`: three independent drafts, fourth-launch
  refusal, protected close, slot reuse, Colors over dirty Notes, cold reopen and
  independent whole-store oracle. Artifacts: `desktop/20261019T225656787818Z`.
  QMP-to-visible-text upper-bound samples: 105.5, 113.7, 111.4 ms.
- `desktop_test.py --deep --trace-storage`: Split + Notes + Colors coexist, nested
  drafts survive switching, repeated close/reclamation, live layout, deeper Split,
  funding refusal and cold reopen. Artifacts: `desktop/20261019T230236259836Z`.
- Launcher recovery `--mode recover` on a disposable copy (automatic retry, fallback,
  explicit retries, draft retention and cold save); other recovery modes were not rerun.
  Artifacts: `launcher-recovery/20261019T230133569587Z`.
- x86 smoke, ARM64 portability, `--ipc-test`, `--matrix-storage-test`, `--session-test`.
  Session also checks the full 32-handle wait and sustained drag (120 reports,
  no lost motion; 31 ms to final observed cursor in that run).

The mixed test exposed the old sixteen-reader metadata allowance despite sufficient
page funding. Forty scene surfaces now have 48 local reader slots, covering normal
replacement/ingress headroom; the independent global read pool remains bounded at
128. Exhaustion probes now use contract capacities rather than stale literals.
Automated images are disposable; the interactive `cathedral-storage.raw` was untouched.


## 2026-10-19 ? Nested desktop and atomic app selection

The ordinary Nested tile is the stock matrix compiled with `nested-desktop`,
not a new platform service. Start at `distribution/matrix/main.rs`, then
`service/children/startup/startup.rs`. It copies finite installed seeds inside its
own realm, prepares missing `Users/Template` names, and runs the same chooser in
picker mode. Its selected user is another stock matrix using the usual CONFIGURE
path. The picker is stopped before that user presents. Inner Notes and Colors
keep saved data under their own program subtrees; there is no ancestor storage
binding, physical input or display endpoint supplied to this program.

The template carries another Nested executable with finite seeds. Preparing a
world is lazy, not infinite eager copying. Existing names and data are never
overwritten; each creation/copy is atomic and retryable, while preparation as a
whole is not one transaction. This is selection of a realm, not Warden auth,
encryption, guest-kernel execution or hardware virtualization.

Bounded funding: Nested requests 320 private pages, 512 reader pages, 40 channels,
five descendants and 24 surfaces. It gives its selected user 256/384 pages,
24 channels and three descendants; that supports chrome, background and one leaf.
Close other outer apps before launching Nested. Another inner Nested level is
present in the chooser but refused with NO ROOM by the current resource ceiling.
No task-count or kernel memory-cap increase was made for this milestone.
Atomic switching exposed the framebuffer aperture overlapping shared mapping
slots above the old sixteen-reader range. Its virtual address is now
`0x0000_0080_0400_0000`; a compile-time assertion keeps the entire guarded object
mapping arena below it. Display clients use DISPLAY_INFO, not a baked-in address.
The normal outer desktop still supports three ordinary leaves. Fixed surface
ranges may refuse fragmented capacity rather than moving live siblings.

CTHIMP04 extends the sealed declaration with page/surface demand and requires all
five requested resource amounts to fit the owner offer before spawn. It uses the
stock matrix's ordered environment entry. Legacy declarations retain their prior
rules. Stock config ceilings now admit larger matrix programs while ordinary
leaves retain 128/128 pages and ten surfaces; old recognized stock configurations
are upgraded without replacing custom layout. The fourth tile puts task rows at
local y=504, and chrome has seven surfaces. The storage deadline is 1200 ticks
(12 seconds at 100 Hz), with client and watchdog waits derived from it. The larger
template copy exceeded the previous 600-tick bound during extent growth in
disposable runs. The boot owner's health wait now covers the matrix storage
deadline, avoiding a root-connection panic during a larger template copy.

App selection stages old/new app visibility in the existing presentation protocol;
its batch capacity now covers the 40-surface scene. Commit publishes the entire
switch; refusal aborts and restores local selection. Startup, teardown, settings
and individual app repaint are still separate operations; this is not vblank sync.

Remaining follow-ups: scalable resource negotiation (including multiple inner
apps/deeper desktop nesting), responsive narrow-screen chrome, nested background
and picker recovery, explicit handling of user-matrix startup failure, automatic
nested layout reflow, and stock-code upgrades inside already-created inner users.
Current installation upgrades outer stock realms and Nested's installed seeds;
existing inner snapshots deliberately retain their code. Reopen after a failed
inner startup by closing Nested from its outer task row. Multiple instances of
Nested share its saved Users subtree, just as multiple Notes instances share their
program data. No implicit per-instance realm clone is made.

Verification:

- Rust workspace host tests and 36 storage-lab engine tests pass; userspace/UEFI
  checks, Nested + trace clippy, formatting, layout (35 Omega / 121 Rust edges)
  and 13 harness unit tests pass.
- `desktop_test.py --nested-desktop --trace-storage` passes inner user creation,
  Notes/Colors persistence across cold reboot, explicit UNSAVED WORK and NO ROOM
  refusals, subtree teardown and outer/inner Notes isolation. Artifacts:
  `desktop/20261019T234725230182Z`.
- Normal desktop passes three Notes instances, switching, refusal, protected close,
  slot reuse, cold reopen and the independent whole-store oracle (532 records):
  `desktop/20261019T234900506491Z`.
- Deep Split passes coexistence, resizing, recursive close, funding refusal and
  cold reopen: `desktop/20261019T235047434065Z`.
- ARM64 portability proof and x86-64 smoke pass after the framebuffer relocation,
  including display restart, aperture guards and mapping rollback/reclamation.
- Matrix desktop passes recursive atomic publication, admission rollback,
  stalled-provider isolation, expired-request refusal, lost-ack reconciliation,
  provider replacement, physical input and cold reopen. Matrix storage passes
  scoped delegation/revocation, copies, realm-loaded programs and replacement.

The fault-injection regression caught a client deadline still fixed at 1200 ticks
after the provider bound was lengthened. `storage::EXCHANGE_TICKS` now derives
from twice that bound, leaving time for the provider verdict to return without
retiring the client connection. The stall fixture now distinguishes an expired
pre-execution request (no write) from a committed request whose reply was lost;
its former expectation that both committed contradicted provider deadline checks.

One earlier disposable normal-desktop run (`desktop/20261019T234336003280Z`)
reported NO_MEMORY during login and recovery-screen publication. It did not
reproduce in subsequent normal, nested or deep runs; cause remains unresolved.
The previous intermittent ATA busy timeout remains open too. Tests use disposable
images; the interactive saved disk was not altered.

## 2026-10-19 - Desktop measurement and collection I/O

The next performance step uses the ordinary F5 composition. Start at
`tools/boot-harness-rs/desktop_test.py`, whose steps feed `desktop_timing.py`;
`storage_timing.py` parses the serialized provider's phases. Each boot records
host boot/login/launch/typing/switch/save observations. Optional `--trace-storage
--trace-input` adds provider CPU, sector counts, collection/commit/extent spans
and compositor redraw CPU. Reports and interpretation are in the harness README.
Host observations include QMP, deliberate input spacing and screenshot polling;
guest ticks, per-process CPU and host wall time are distinct clocks. Tracing
perturbs execution. These are diagnostic observations, not performance promises.

The main measured save cost was reachability collection: repeated per-chunk
index lookups across shared program manifests thrashed the small engine cache.
`platform/libraries/database-objects/collection.rs` now marks distinct referenced
index keys, validates them in one ordered metadata scan, then performs the same
bounded removals. Missing indexes still refuse collection or pinning before any
mutation. Pin validation also uses this scan, trading individual lookups for a
root-wide ordered index walk when the pinned object has chunks. Metadata work
still scales with the live set; at most eight removals does not bound total
collection CPU to eight records. No storage format, durability, retention or
kernel authority changes were made.

The provider cache is 64 KiB instead of 32 KiB, inside its existing 56-page heap.
A 128 KiB trial exhausted that heap while copying a user and was discarded.
Ordered validation with 32 KiB also reduced reads, but 64 KiB further improved
the measured working set. This is a bounded tuning choice, not general memory
negotiation or proof that every future workload fits.

Comparable `--trace-storage` runs (artifacts under `build/boot-harness-rs/desktop`):

| Observation | Baseline `20261020T002203005756Z` | Optimized `20261020T002630278955Z` |
| --- | --- | --- |
| Notes save host upper bound | 1506.6 ms | 358.4 ms |
| Notes provider sectors read | 2785 | 201 |
| Colors save host upper bound | 3053.4 ms | 525.9 ms |
| Colors provider sectors read | 5410 | 194 |
| Create user host upper bound | 3117.6 ms | 2045.1 ms |

Host load varied. Login and launch did not show a corresponding improvement;
do not generalize these save results to all desktop latency. The final run with
both trace features (`20261020T002959149665Z`) measured about 2-3 ms compositor
CPU per typed character, within 132-136 ms host observations. This does not
attribute all remaining latency to scheduling: app rendering, IPC and fixture
overhead are not separately resolved by these spans.

Verification:

- Rust workspace host tests; 16 database-object tests with `--features host`,
  including shared chunks/pins, corrupt-index refusal without mutation, existing
  cold-write recovery tests and a cold 64 KiB-cache collection I/O regression;
  all 36 storage-lab engine tests pass.
- Userspace clippy, display input tracing and storage trace/recovery feature
  clippy, formatting, layout (35 Omega / 121 Rust edges), and 17 harness unit
  tests pass. No kernel/ISA code changed; ARM64 was not rerun for this milestone.
- Ordinary desktop with both traces passes three app instances, input, switching,
  saves, protected close, cold reopen and the independent whole-store oracle
  (536 records): `desktop/20261020T002959149665Z`.
- Nested desktop passes both boots, inner Notes/Colors persistence, recursive
  dirty-close refusal, funding refusal and outer isolation:
  `desktop/20261020T003211305619Z`.
- `run.py --matrix-desktop-test --release --timeout 120` passes stalled-provider
  isolation, expired request refusal, lost acknowledgement reconciliation,
  replacement, focus/clipping and cold reopen; artifacts in `matrix-desktop-test`.

Next: resource accounting and funding negotiation, then nested background/picker
recovery and layout reflow. Keep timing evidence alongside those changes rather
than raising capacities blindly. Initial-user creation and login remain visible
costs worth separating further. Previous unresolved NO_MEMORY and intermittent
ATA reports remain open; the deliberately oversized cache trial above is a
separate, reproducible and reverted experiment. All guest runs used disposable
images; the interactive `cathedral-storage.raw` was untouched.

## 2026-10-19 - Live resource growth through matrix owners

Start at `distribution/matrix/service/children/admission.rs`: resolve imports,
check surface placement, ask for a local funding shortfall, then reserve/spawn.
`children/funding.rs` authenticates a requesting child's endpoint, applies stock
ceilings and obtains its own shortfall upstream before transferring funds. Shared
amount/ceiling checks live in `platform/libraries/matrix/funding.rs`; the platform
`FUND` packet is on the supplied interaction connection. The kernel knows neither
matrix roles nor this protocol.

`RESOURCE_GROW` (71) atomically transfers increments of private pages, reader
pages, channels, free descendant slots and physical frames along an owned launch
edge, including a live child. The syscall validates the complete input before
touching ledgers; the resource entrance validates all five families before any
commit. Runtime mapping tables already cover the fixed mapping arena. This does
not enlarge global pools, mapping/handle limits or CPU allowances. A separate
`NO_PROCESS_CAPACITY` refusal identifies unavailable transferable slots, including
the existing ordering constraint, without revealing account identities.

Current stock growth policy is explicitly bounded: programs admitted with
descendants can grow to the larger of their initial grant or
384/768 pages, 40 channels, six descendants and 1792 physical frames. Ordinary
leaves have no growth policy. Each transfer is atomic; the complete ancestor chain
is not one transaction. Capacity granted to an intermediary may remain there
after a downstream refusal and is reused or returned when that subtree closes.
There is no live shrink/recall protocol or automatic retransmission after a lost
reply; a failed transport retires the upstream connection.

The ordinary Nested program now requests 32 surfaces, still from the existing
40-surface scene. Its fresh user runs Notes and Colors concurrently. A third leaf
or another Nested still refuses. Surface ranges remain fixed, and the global
15-task ceiling remains unchanged. The initial simultaneous-app test exposed a
stranded picker account below the selected user's process-order boundary. Nested
now loads user code, retires the picker, reserves the user (reusing that account),
then publishes its full view. Admission refusal restarts the picker. A later
user-startup fault still needs the broader recovery work listed below.

Refusal cleanup also exposed an unused object-executable reference: the old path
minted it before policy checks and could retain it on reservation refusal.
`TASK_IMAGE_RELEASE` (72) now drops an unused caller-local object grant; templates
are refused and consumed grants are stale. Stock admission creates the grant only
after preflight and releases it on reservation failure. This preserves the
existing reservation rollback after a failed spawn.

Trace diagnostics distinguish page/reader/channel/process/frame shortfalls,
policy ceilings, transfer failures, surface range exhaustion and local app slots.
The desktop keeps NO ROOM. In the nested run, the inner matrix requested
`[100,0,0,1,97]`; its owner obtained `[36,0,0,0,0]` from the outer user, then granted
the full inner request. No resource inventory crossed a child-facing boundary.

Verification:

- Workspace host tests pass, including five-family atomic refusal with live
  charges and platform ceiling/overflow tests. Userspace/UEFI clippy, layout
  (35 Omega / 121 Rust edges) and 17 harness unit tests pass.
- IPC QEMU passes live page growth/use, invalid-address/handle refusal, subtree
  cancellation and budget restoration. Its executable probes drop more than one
  grant-table's worth of abandoned grants and verify stale names plus exact quota,
  process and physical baselines.
- Concurrent Nested Notes/Colors, repeated refusal, switching, reuse after close,
  isolation and both cold boots pass: `desktop/20261020T030752847109Z`.
- Normal desktop, three independent Notes drafts and whole-store oracle pass:
  `desktop/20261020T030423386464Z` (536 records). Deep Split passes coexistence,
  nested close/resize/funding refusal and cold persistence:
  `desktop/20261020T030320712540Z`.
- Matrix desktop recovery and cold reopen pass. ARM64 portability proof passes;
  the new live-growth syscall's guest exercise is currently x86-64.

Next: nested background/picker and post-admission user-startup recovery, live
layout reflow, and a deliberate upgrade route for saved inner code snapshots.
Existing inner users retain their code; create a new inner user to try live
growth. Flexible surface funding and live quota return are still separate work.
Previous intermittent ATA and unreproduced NO_MEMORY reports remain unresolved.
All tests used disposable images; the interactive saved disk was not touched.

## F5 live quota and transaction headroom repair (2026-10-19)

The owner's saved image reproduced `live/pinned content exceeds root quota`
before QEMU launch. There were two separate problems: labels checked admission
before named replacements, and the complete upgraded live/pinned set genuinely
needed 1,287,797 bytes, beyond the old 1 MiB allowance. Existing nested users keep
their older code snapshots; they cannot be silently discarded as upgrade garbage.

The coordinated installer now stages labels without collection/admission, then
finishes hierarchy and named programs before collecting unreachable history and
checking final quota. Standalone label/hierarchy migrations still enforce quota.
Hierarchy checks even when its own plan has no changes, and no longer rejects the
intermediate plan before named replacements. Final excess reports required bytes
and leaves program publication untouched.

The logical root allowance is 2 MiB. Merely raising it exposed a second failure
on the owner's fuller image: it booted, but the next Notes save ended the provider
with `Engine`. The compacted database occupied 2,609,152 bytes and the old 4 MiB
extent did not leave enough transaction allocation space. Database lab media and
the provider now use an 8 MiB assigned extent. F5 recognizes legacy guarded 4 MiB
images and invokes explicit offline `grow` before installation, even if the
program receipt already matches. Growth holds the image lock, syncs a complete
content-named backup, prepares and cold-compares every record off-image, then
grows the file and moves its trailing guard. Identity and contents survive. It
can restore the original length/bytes on ordinary publication errors. Like the
existing offline compactor, interruption during publication requires its backup;
this is not a power-loss-atomic resize. A successful growth may remain if a later
program upgrade refuses. No existing media is reformatted.

Verification:

- Workspace host tests, all 16 database-object host tests and all 38 storage-lab
  engine tests pass. Regression coverage crosses quota with mixed generations,
  admits the final replacement, preserves user/custom heads, and refuses pinned
  final excess with byte-identical input. Offline growth checks records, identity,
  guards, backup, shrink refusal and refusal before publication.
- Quota-pressure fixtures scale with the allowance. The object quota fixture now
  uses unique chunk identities rather than wrapping byte values after 251 chunks.
- All 19 harness unit tests pass, including growth with an unchanged receipt and
  refusal without replacing media/receipt. Userspace, storage-lab and object-host
  clippy, workspace formatting and layout checks pass.
- Default desktop creation, concurrent Notes drafts, close, saves, cold reopen
  and the 536-record whole-store oracle pass on 8 MiB media:
  `desktop/20261020T034543188251Z`.
- The exact saved disk was tested in `quota-upgrade-repro`: 77 heads survive;
  only nine recognized stock program heads change. It boots to the existing
  users, enters User 1, launches Notes, saves `quotacheck`, and cold-opens that
  text with healthy storage. Test text was written only to a disposable copy.
- The interactive `cathedral-storage.raw` and receipt have now been upgraded.
  Its complete tables/records, identity, guards and even physical bytes match the
  verified upgraded copy. The original content-named backup ends in
  `be036e6641b95064e652fb60da2483aaab47ec14f01cf8ceae620eee9eea8387.raw`;
  the old receipt is also kept in `quota-upgrade-repro/interactive-before.programs`.

These remain fixed lab capacities, not a general quota/volume expansion policy.
Saved inner-user code snapshots are still deliberately retained. The previous
live-growth milestone's recovery/layout follow-ups remain next; no new owner
semantic decision was needed for this launch repair.


## Compact taskbar and bounded recursive desktop (2026-10-19)

The stock launcher occupies an 80-pixel bottom bar with 48-pixel app icons,
horizontal running-instance buttons and right-side controls. Defaults use eight
pixels of margin/gap. Login remains full-screen. Matrix configuration writes
CTHMAT02; CTHMAT01 decodes to compact geometry while preserving launcher selection,
key authority and resource fields. Saved version-2 arrangements remain unchanged.

The bounded kernel arena has 31 accounts, 64 boot endpoints, 192 dynamic channels,
512 sealed reader handles and a maximum 4096 reader-page grant. Both architecture
heaps are 2 MiB. Memory backing vectors initialize directly in their heap arena;
the kernel exhaustion probe keeps bookkeeping in its allocations. The enlarged
spawn record is aligned to 512 bytes so checked single-page copy-in remains valid.
IPC escrow retirement uses a word array, tested across every 64-bit boundary.
Abandoned executable references force a return to the boot loop for reclamation
before the next caller budget query. Their pending counters cover a whole subtree;
the admission guest checks that physical and grant budgets return to baseline.

The display/matrix surface ceiling is 64. Admission may lend descendant hosts
up to 24 spare ranks above their declared minimum, within existing disjoint ranges
and standing ceilings. Funding keeps 256 reader pages at forwarding owners for
code and image replacement. These changes conserve authority; manifests do not
mint budgets. Presentation batches reset in place to avoid stack-size-dependent
teardown faults while dropping all retained readers.

Nested startup recovery is local: one automatic picker retry, then exit to the
outer taskbar; failed selected-user startup restores its picker; one background
retry, then a usable desktop without a background process. Outer drafts/siblings
survive repeated attempts. Saved inner executable/template snapshots are still
intentionally retained. Use a fresh outer user for new code at every depth; a
managed recursive upgrade policy is separate work.

Storage distinguishes retained quota refusal from known pre-commit extent refusal.
For the latter it drops/reopens the poisoned engine before returning capacity,
keeping authenticated connections. Commit failures remain uncertain and other
I/O/corruption failures still retire the provider. Opening an existing object table
uses no writer. The ATA flush deadline is 250 ticks (other phases remain 50), inside
the existing request deadline; there is no automatic retry of a failed write.

Verification completed during this milestone:

- Workspace host tests, 17 database-object host tests, 38 storage-lab engine tests,
  19 harness unit tests, userspace/UEFI clippy and layout checks pass.
- Ordinary desktop, three simultaneous Notes drafts, fourth-launch refusal,
  close protection, reuse and cold reopen: `desktop/20261020T043449182055Z`.
- Settings cancel/live geometry/cold persistence: `settings/20261020T043606799710Z`.
- Launcher crash recovery and startup fallback: `launcher-recovery/20261020T043733935425Z`.
- Full retained quota, three save refusals, failed close-save, surviving draft,
  sibling reads and two boots; independent oracle confirms every record unchanged:
  `storage-capacity/20261020T043905119323Z`. Physical extent poison/reopen and cleanup
  retry are separately tested against the real redb engine on the host.
- Nested picker, selected user and background fault injection, repeated cleanup
  and surviving outer Notes draft: `nested-failures/20261020T044335198301Z`.
- Three Nested desktops beneath the outer desktop, deepest Notes save/read,
  recursive dirty-close protection, two subtree close/re-enter cycles, isolation
  and all checks repeated after cold reboot: `desktop/20261020T044754085311Z`.
- Deep Split coexistence, drafts, resize, nested close, funding refusal and cold
  persistence: `desktop/20261020T044435628405Z`. Management original/recovered/revoked
  authority cases pass: `management/20261020T044828618328Z`.
- Provider live pre/post-commit recovery: `provider-live/20261020T044643905697Z`;
  startup crash-once/bad-volume: `provider-startup/20261020T045010333443Z`;
  all four lifecycle cases: `lifecycle/20261020T045034245325Z`. Other provider
  fixture variants were not rerun in this milestone.
- Matrix desktop, admission and IPC guest suites pass, including executable
  release and reclaimed resource baselines.
- Full current F5 installation on a copy of the owner's two-user saved disk,
  Colors launch/save/close/reopen and cold boot: `colors/20261020T045258562388Z`.
  The upgraded store contains 1188 records and fits the existing assigned extent.
- ARM64 portability proof and x86-64 smoke pass; smoke used a 90-second host timeout
  under concurrent load after the default 30-second run timed out late in the suite.

Deep startup remains a performance follow-up: the successful run observed up to
55 seconds at the third level. An earlier run exceeded the old 45-second scene
wait; nested checks now allow 120 seconds and report time at each depth. This is
an observation bound, not a guest deadline change or a latency fix. Concurrent
desktop tests also exposed shared profile artifacts overwriting each other's
installed programs before cold verification; each run now has its own profile
and program paths. The isolated three-level rerun passed both boots.

Next: profile recursive startup and steady-state rendering/IPC costs, deliberate
upgrades for saved inner code snapshots, and flexible surface funding/live quota
return. No interactive saved disk has been modified during this milestone; F5
will install the new stock code normally. A fresh outer user is needed to get
the current implementation throughout the Nested template chain.

## Saved Nested user launch ABI repair (2026-10-19)

The owner's saved Nested user returned CANNOT OPEN after the task-capacity
increase. Reproduced on an unchanged copy of the interactive disk:
`nested-saved-repro`. The old user matrix reached startup but its launcher spawn
returned INVALID_ARGUMENT. Raising MAX_LAUNCHES from 14 to 30 had also enlarged
TASK_SPAWN_TREE's implicit request from 176 to 304 bytes. The kernel read unrelated
bytes beyond the old request and rejected them as nonzero unused grant entries.
Fresh-image tests rebuilt every executable together and missed this compatibility
failure; preserved code must also be exercised against a new kernel.

MAX_SPAWN_LOANS now fixes that syscall's wire capacity at 14 independently of the
31-account kernel arena. The kernel and runtime enforce the same limit. Dynamic
descendant reservations and the three-level desktop capacity remain unchanged.
No executable snapshots, configuration or user data need replacement for this fix.
Saved desktops continue to use their saved UI; fresh users get the new taskbar.

Verification: host tests, userspace/UEFI clippy, layout and format checks; IPC guest
suite including a real recursive spawn with nonzero adjacent bytes after the old
176-byte record; admission guest suite. An unchanged saved-disk copy with the new
kernel reaches the inner desktop (`nested-saved-fixed`), where the original
vertical launcher renders. A second boot from another unchanged copy also launches
inner Notes (`nested-saved-fixed-app`). The interactive saved disk was not modified.

## Fresh lab state instead of recursive legacy upgrades (2026-10-19)

Owner direction: legacy lab users do not need compatibility or migration; making
new users is cheap. This supersedes the earlier proposed recursive snapshot
upgrade milestone. Current stock seeds already produce the bottom taskbar at
every depth. Cached inner templates on the retained disk explain the old UI.

The default launch still keeps session data. The new **Cathedral: Fresh Lab in
QEMU** launch entry adds `--fresh`. After building, it sets the disk and receipt
aside together, then uses normal first-install seeding. It refuses a busy disk;
a failed receipt move restores the disk. No kernel/platform changes or recursive
program migrations were added. Use this route when discarding obsolete lab state.

Fresh creation verified all three taskbars. Cold reopen then exposed the chooser's
hard-coded 600-tick LAUNCH receive timeout, shorter than the matrix's bounded code
load/funding work. It panicked while selecting the third nested user. LAUNCH now
uses a named stock-contract budget of eight storage intervals plus reply margin,
like multi-stage CONFIGURE. Other UI requests retain their short deadline; no
operation is automatically replayed after an uncertain timeout.

The interactive disk was reset through this command. Its prior disk and receipt
are in `build/boot-harness-rs/before-fresh-rc75aito/`; the fresh disk contains only
current stock seeds, ready for NEW USER. Validation: 21 harness tests, Rust host
tests, userspace clippy, formatting and layout checks pass. A copy of this newly
seeded interactive disk passes fresh user creation and bottom-taskbar rendering
at all three Nested levels, then repeats those checks after cold reboot:
`build/boot-harness-rs/fresh-taskbars-fixed/`. The test's users are only on that
copy. This focused check does not repeat the previous milestone's full subtree
close/exhaustion suite. Deep startup remains slow. Focus subsequent work on actual
OS behavior, not legacy lab upgrades.

## Nested storage request amplification (2026-10-19)

The latency workload now lives in `tools/boot-harness-rs/desktop_latency.py`,
entered through `desktop_test.py --latency --trace-storage --trace-input`.
It builds the ordinary F5 composition on a disposable disk, measures the outer
desktop and three Nested levels, saves at each depth, then cold-opens the same
disk. Picker launch, user creation/entry, Notes launch, pointer, typing, save
and close have separate observations in `boot-*-latency.json`. The existing
`desktop_timing.py` reports provider operations, sectors and compositor CPU.

The two-slot scope packet was causing repeated ancestry HEADs at each ancestor.
At eight forwarding hops, the host model expanded one write into 21 provider
operations. `platform/libraries/matrix/world/storage` now retains four successful
containment proofs per binding, reducing a repeated write to one provider call.
Stock and Split confirm proofs only after the matching upstream check succeeds.
Requests still authenticate the live connection/revision and reach the provider
with their innermost scope. Proofs never authorize an operation or cache its data.
Immutable parents and never-reused object numbers are essential to this rule.
Revocation, regrant and upstream changes invalidate proofs; retiring/replacing a
root provider revokes **all** its bindings, leaving other sources independent.
Eviction requires another proof. No kernel mechanism or IPC wire format changed.

The stock nested seed copier also checks whether a destination exists before
looking up its source. This removes ten unnecessary source lookups on reopening
an existing template, without replacing its programs or data.

Measured third-level observations (seconds, traced QEMU/TCG):

| Phase | Before | After | Provider calls before → after |
| --- | ---: | ---: | ---: |
| Fresh picker launch | 41.62 | 21.41 | 449 → 60 |
| Fresh user entry | 27.40 | 7.91 | 312 → 27 |
| Cold picker launch | 26.23 | 6.34 | 283 → 28 |
| Cold user entry | 40.22 | 7.83 | 312 → 27 |
| Cold Notes launch | 13.43 | 3.36 | 89 → 9 |
| Cold Notes save | 5.20 | 3.00 | 21 → 1 |

Baseline artifacts: `desktop/20261020T054908071290Z`; revised artifacts:
`desktop/20261020T055706461711Z`, both beneath `build/boot-harness-rs/`.
The baseline cold workload was rerun against its original ESP/disk after fixing
a fixture mistake: retyping identical saved text makes Notes clean, so the cold
workload now types different text and checks each deletion. Fresh baseline data
is from the original completed first boot. Both corrected cold workloads passed.
Host observations include QMP/screenshot overhead, host load varies and tracing
adds work. These are observations, not timing promises; operation counts show
the eliminated work independently of host scheduling.

This does **not** establish an input latency improvement. Revised third-level
typing observations span 187–335 ms across both boots; root pointer observations
span 40–136 ms there. The compositor CPU for the three cold typed characters was
about 2–6 ms each, measured on its own clock. Do not subtract it from host time
and label the remainder scheduler or IPC cost.

Next measured targets: first-time seed creation still takes 21 seconds at depth
three; a single cold save still reads 938 sectors and writes 146, with about two
seconds of provider-account CPU in this run. Investigate collection/commit/read
work and serialized seed creation before blaming rasterization or adding SIMD.
Separately measure chrome polling and interaction round trips; their costs have
not yet been isolated. No new owner decision is needed for this work. The
interactive disk was not reset or modified during these measurements; Fresh Lab
remains the way to discard obsolete executable snapshots.

The subtree-reopen fixture also needed a stronger close observation. A blank
canvas appears before child collection, and all nested backgrounds use the same
color. Its old one-pixel assertion could pass with the subtree still alive;
the next launch then exhausted surface funding. Both nested/latency workloads
now wait for the owner's running-task cell to clear too. The first full-tree
run (`desktop/20261020T060137243348Z`) hit that sequence and ultimately ended in
a host Python `MemoryError` during screenshot capture, not a guest panic.

Verification completed: Rust workspace host tests (including real recursive
proof forwarding, stale bindings, forged scopes, source invalidation and bounded
eviction), 21 Python harness tests, default userspace clippy and the stock matrix's
nested/trace/fault/probe feature set, formatting and layout checks. Formatting
also normalized a few pre-existing named Rust bodies; kernel edits are cosmetic.
The latency workload passed both boots. Provider replacement, uncertain commits,
independent roots and cold verification passed in
`matrix-database/20261020T060012433920Z`. Two-level Split persistence, protected
close, funded refusal and repeated reclamation passed both boots in
`desktop/20261020T060035553703Z`. ARM64 and kernel smoke were not rerun for this
userspace change.
The corrected three-level desktop fixture passed both boots, deepest saved
Notes, protected close, two complete subtree teardown/reopen cycles per boot,
and an isolated outer note in `desktop/20261020T060627976276Z`. No guest code
change was needed for that fixture race; waiting for actual task removal fixed
the observation. These later untraced runs experienced different host load and
are functional verification, not additional before/after performance samples.

## Storage maintenance and nested interaction costs (2026-10-20)

The previous traced cold third-level save spent 1,601,597 us of provider CPU in
collection and 262,889 us in its publication commit. The provider was scanning
every live head/manifest before every small mutation, including each template
copy. `Store::collect_under_pressure` now reads the durable quota ledger first:
below 1.5 MiB it does no writer/collection scan; at or above that watermark it
runs the same eight-record bounded pass. The 512 KiB reserve is maintenance
headroom, not a new admission rule. Every mutation still checks exact quota,
and explicit COLLECT remains unconditional. There is no automatic retry of an
uncertain write. Pins, immutable content, immediate durability and the on-disk
format are unchanged. Unpinned old revisions can remain longer; they have no
retention guarantee. The independent small-workload host oracles now expect
that retained history rather than assuming every request collected it.

Nested realm preparation also resolves each named source at most once in that
preparation. Workspace and next-level template copies share those local
source/head results. Destination checks still precede lookup; nothing survives
the preparation, and each COPY still validates the expected source revision.
This removes ten provider requests from fresh third-level picker setup, from
60 to 50, without a new batch protocol, authority shortcut or partial-tree
publication rule.

Notes now retains its unchanged background, close button and tab bitmap.
Subsequent ordinary text edits publish only a replacement text bitmap. Successful
decoration updates are remembered within one scene, and invalidation discards
the scene so every surface is recreated. The `trace-input` route now includes
Notes key receipt and paint begin/end. Per-step JSON records the ordered
source/matrix/Notes observations; source and matrix releases are excluded from
the key-delivery series. Different matrix CPU counters must never be joined.
An incomplete paint span can cross a screenshot observation boundary, so do not
pair an old end with the next key's begin.

Traced QEMU/TCG observations at the third nested level:

| Measurement | Previous milestone | Revised |
| --- | ---: | ---: |
| Cold save observed time | 2998 ms | 739 ms |
| Cold save provider CPU | 1986 ms | 391 ms |
| Cold save collection/ledger CPU | 1602 ms | 18 ms |
| Cold save sectors read / written | 938 / 146 | 217 / 97 |
| Cold save durable flushes | 2 | 1 |
| Fresh picker observed time | 21410 ms | 16323 ms |
| Fresh picker provider requests | 60 | 50 |

Baseline: `desktop/20261020T055706461711Z`. A pressure-only intermediate run
with input instrumentation is `desktop/20261020T155022438947Z`; it completed both
boots before the Notes/seed changes. The completed revised workload is
`desktop/20261020T155343691135Z`. All paths are under `build/boot-harness-rs/`.
Host load and trace overhead vary; counts and eliminated work are stronger
evidence than a timing ratio. Cold user entry still took 8.3 seconds and fresh
entry 11.5 seconds in the revised run; these phases were not fixed.

Deep typing observed 209–240 ms fresh and 114–118 ms cold, so responsiveness is
still uneven. Source-to-Notes delivery crossed seven matrix processes and took
5–8 nominal 100-Hz guest ticks in the revised samples. For one cold character,
delivery took six ticks, Notes paint took five ticks including IPC, Notes charged
about 1.8 ms CPU during that paint, and compositor redraw about 0.9 ms. These
separate clocks are not a decomposition of host elapsed time. The useful next
measurement is wake/dispatch and interaction round trips across the chain,
including chrome status polling; do not assume rasterization or assign all
remaining time to kernel scheduling.

Remaining storage limits: a pressured root still pays for a whole live-metadata
scan; physical engine capacity is independent of logical quota; creation still
does multiple durable transactions. The revised deep save still reads 217
sectors for a tiny payload. Investigate scope walks, engine reads/cache and
commit allocation separately before increasing cache/heap funding or adding a
general transaction API. Kernel scheduling and ABI are unchanged in this work.
No new owner question blocks these follow-ups. The interactive disk was not
modified or reset; use **Fresh Lab** for updated programs at every nested level.

Verification:

- Workspace host tests, 18 database-object host tests (including a cheap
  cold-cache ledger check, bounded pressure collection, pin survival, exact
  accounting and unchanged independent roots), 38 storage-engine host tests and
  22 Python harness tests pass. Default userspace and changed trace/fault/probe
  feature clippy checks pass, as do layout and Rust format checks.
- The default F5 desktop passes fresh/cold operation, three Notes instances,
  draft switching, close/save/discard, tabs and the independent 541-record oracle:
  `desktop/20261020T155644324318Z`.
- The standalone object fixtures had two stale assumptions: their 4 MiB image
  no longer matched the 8 MiB provider, and reader-page exhaustion expected BUSY
  instead of the kernel's NO_MEMORY. The volume seeder now accepts a size while
  retaining its original default for the separate volume-IO lab; the object
  fixture uses the database extent and still verifies outside guards. The
  pressure assertion still requires reads to resume after readers are dropped.
  Two-boot publication/refusal/cold recovery passes at
  `objects/20261020T155909026335Z`; three-boot pins, bounded collection, bulk
  pressure, exact cold records and final heap/reader release pass at
  `objects/20261020T160023934011Z`.
- Provider replacement, ambiguous commits, independent roots and exact cold
  oracle pass at `matrix-database/20261020T160057846155Z`.
- Two forced Notes rendering invalidations during creation/publication retain
  the draft and restore input: `settings/20261020T160125588429Z`.
- Full retained quota repeatedly refuses save and close-save while preserving
  drafts, sibling reads and all persisted records across two boots:
  `storage-capacity/20261020T160241919815Z`.
- Three nested desktops, deepest saved Notes, recursive dirty-close protection,
  two subtree close/reopen cycles per boot, confinement and cold recovery pass:
  `desktop/20261020T160310046094Z`.
- No kernel behavior changed; ARM64 and kernel smoke were not rerun.

## Continuous pointer movement and nested matrix exits

The owner reported APP STOPPED followed by losing inner desktops during normal
F5 use. A fresh disposable desktop reproduced a matrix exit while creating a
nested user with pointer motion during every scene wait. The first reproduction
is `desktop/20261020T162724840470Z`; the instrumented reproduction is
`desktop/20261020T163058173648Z`. The latter identifies `POLL` (operation 5)
timing out with `-110`, followed by `matrix-service-exit`. This is a userspace
matrix failure, not an observed kernel fault or an old-disk-only failure.

The stock matrix used to drain up to eight upstream input events at the start
of a turn, then refresh again before dispatching the selected child request.
Each ancestor could do the same before answering that child's POLL. Under
continuous motion this delayed nested replies past the unchanged interaction
deadline. The parent now consumes one snapshot per service turn. MORE records
remaining work and makes the next idle wait immediately runnable, while an
already-selected child request gets its turn before another refresh. Quiet
notification-driven matrices still park; legacy two-endpoint fixtures still
refresh on demand. No scheduling grants or IPC deadline were increased.

The first fairness-only stress run completed its fresh boot but lost a deepest
Notes close click during a cold reopen (`desktop/20261020T163320913383Z`);
there was no matrix exit. Bounded child queues now coalesce trailing pointer
motion within a button state, retaining the first report's coordinates and the
latest motion. Button transitions, keys and cancellations remain ordered.
Non-coalescible overflow still cancels only that child's queue. Host regressions
cover a motion flood around press/release, key/cancel boundaries, and sibling
isolation on genuine overflow.

`desktop_test.py --nested-desktop --pointer-stress` moves the pointer throughout
startup/scene waits and for three seconds after each desktop becomes visible.
It fails immediately on any matrix service exit, and retains the three-level
Notes, dirty-close, recursive reclamation and cold-persistence checks.
Synchronous interaction receive failures now log the operation and status.
Normal F5 also saves serial output to `build/boot-harness-rs/interactive/serial.log`
while still printing it to the terminal. The next boot replaces that log.

The owner's saved interactive disk has not been reset. Use **Cathedral: Fresh
Lab in QEMU**, then create users, to get the corrected executable throughout the
tree; existing user realms intentionally retain their copied programs.

Verification:

- The complete motion stress workload passes on fresh and cold boots, with
  three nested desktops and two subtree close/reopen cycles per boot:
  `desktop/20261020T163904829764Z`.
- Workspace host tests pass, including all 39 matrix tests. The 22 Python
  harness tests, default userspace clippy, stock matrix nested/trace/probe/fault
  feature clippy, layout and Rust format checks pass.
- A disposable headless boot using the interactive stdio chardev reaches the
  user picker with the readiness marker in both terminal output and the live
  serial file: `interactive-log-check/20261020T164355066761Z`.
- Ordinary desktop fresh/cold behavior and the independent 541-record storage
  oracle pass at `desktop/20261020T164622379795Z`. An earlier attempt at
  `desktop/20261020T164407521583Z` lost its host Python process without a
  traceback or test result, leaving its QEMU child alive; that owned orphan was
  stopped before the successful rerun. No guest panic or matrix-exit marker was
  recorded in the interrupted attempt; the host termination cause is unknown.
- The existing headless matrix protocol test passes (`matrix-test`): identical
  worlds, local views, attenuated input and recovery over the legacy connection
  shape. No kernel behavior changed; kernel smoke and ARM64 were not rerun.

## Slow interaction replies preserve live matrices

The owner's next normal F5 run still lost inner desktops after the fairness
change. The captured live log is
`build/boot-harness-rs/interaction-delay-owner/serial.log`: a Background unwrap
panics with `-110`, followed by `matrix-service-exit`. Fair dispatch reduced one
cause of delay but did not fix treating a receive deadline as fatal process loss.
The desktop still uses one emulated CPU, 128 MiB and TCG; this repair does not
change those resources, the kernel scheduler or the kernel ABI.

The stock matrix now progresses one asynchronous upstream `POLL_TAGGED` through
its service loop. A connection-local, non-reused request ID associates each
reply with its pending snapshot; it grants no authority. Mismatched
replies are ignored. After the diagnostic threshold it retains its view, focus
and input ordering while waiting for that same reply and serving local child queries.
Deferred mutations stay bounded per stream. Separate storage work progresses;
completed program reads wait for interaction ownership before child admission.
Ready deferred requests precede another poll; requests blocked on independent
storage do not prevent consuming an interaction wake.

Asynchronous presentation also retains its original transaction and readers
across delay. Other serialized interaction calls, image uploads, physical input
forwarding and service acquisition keep waiting for their original response
instead of exiting or retrying the operation. None of these changes retries
ambiguous mutations. Storage transaction deadlines and reconciliation are
unchanged. Expired diagnostic timers leave the asynchronous wait set.

The first five-second-stall run survived all three delays but caught a clean
subtree close being refused during an unrelated poll
(`desktop/20261020T170942625177Z`). Simply queuing LOSS behind that poll still
refused a later clean close under load (`desktop/20261020T172247533354Z`).
Read-only loss surveys now progress on their child control links during polling,
provided no completed admission is waiting. Publication still defers them.
A close reply stays queued until teardown can publish erasures; its ready
receiver is excluded while teardown is blocked. Dirty, missing and uncertain
replies still cannot authorize discarding work. Busy surveys, uncertain child
answers and refused close now have distinct diagnostic traces.

The next run passed its fresh boot but lost a Backspace after the cold boot's
delay sequence (`desktop/20261020T171509381534Z`). Clearing focus on a diagnostic
timeout manufactured an input discontinuity despite retaining the original
ordered reply. Lateness now preserves focus; real upstream cancellation and
queue overflow still cancel input. The test-only late-poll hook wakes children
to prove local queries can complete during the held upstream request.

A later loaded run lost three letters from the initial four-key entry before
any injected delay (`desktop/20261020T172943269969Z`). A host regression confirmed
that a click plus four press/release pairs overflowed the eight-entry child
queue before draining. The queue now stores only the four event fields instead
of full eight-word reply packets, fitting sixteen events in the same 512 bytes.
Replies reconstruct their status, current epoch and padding; grant changes
already clear queued input before advancing the epoch. Motion coalescing and
bounded overflow cancellation retain their ordering and sibling-isolation tests.

`desktop_test.py --nested-desktop --pointer-stress --interaction-delay --timeout 240`
compiles test-only F8/F7/F6 gates at the root. They retain a poll, image upload
and compact publication for 500 guest ticks each. The poll gate first sends an
unrelated DENIED reply; the test requires rejection of that reply, child polling
during the delay, eventual recovery, preserved Notes contents and dirty state,
Colors publication, clean close, subtree reclamation and cold persistence.
Ordinary F5 has no gates. The interactive disk was not reset or changed; use
**Cathedral: Fresh Lab in QEMU** for corrected executables throughout copied
user realms.

Remaining scope: only polling has explicit wire request IDs. Other interaction
operations serialize one outstanding request; this is not a multiplexed RPC
transport. Startup and individual surface exchanges can still block dispatch,
and a live provider that never responds can leave them waiting indefinitely.
Terminal transport errors and malformed replies remain failures. Further work
should make those exchanges resumable and provide recoverable unavailable state
without losing drafts; SMP or larger budgets do not replace that work.

Verification:

- Workspace host tests pass; the matrix suite now has 41 passing tests, including
  tagged replies and the burst regression (observed failing before queue compaction).
  All 22 Python harness unit tests pass. Default userspace and
  matrix nested/delay/trace/probe/fault feature clippy checks pass, as do layout,
  Rust formatting and whitespace checks.
- Ordinary F5 fresh/cold behavior, three independent Notes drafts, close/save/
  discard, Colors and the independent 553-record storage oracle pass:
  `desktop/20261020T173847824501Z`.
- Three nested desktops under continuous pointer motion pass on fresh and cold
  boots with all three five-second delays, mismatched poll rejection, local
  polling during the stall, uninterrupted keyboard focus, draft protection and
  two subtree close/reopen cycles per boot:
  `desktop/20261020T173552869333Z`.
- The legacy two-endpoint matrix protocol fixture passes (`matrix-test`).
  Provider replacement, ambiguous writes, independent roots and exact cold
  storage records pass at `matrix-database/20261020T172005490015Z`.
- No kernel behavior changed; kernel smoke and ARM64 were not rerun.


## Scheduler timing preparation and periodic contention (2026-10-20)

Baseline: `e94a9ef0`. The shared scheduler now maintains active ownership
branches instead of rebuilding readiness over every reserved account. Each
selection visits active direct-child contenders along its descent and preserves
the previous slot-ordered rotation. State/gate changes update the affected
ancestor chain. Local exhaustion hides a whole subtree without rewriting every
descendant; reattachment, restart and slot reuse preserve rotation and debt.

CPU accounting charges only the running ancestor chain and accounts whose
repayment deadlines are due. Idle credit refills lazily, including before a rate
change; running refill/consumption remains one net operation. Indexed heaps hold
one deadline per sleeper/exhausted account, with explicit removal on sleep
cancellation. Ready/finished queries use maintained state. Storage is allocated
at construction, with no allocation in dispatch/accounting/wakeup. This adds
linear metadata per reserved slot in exchange for removing dormant-slot scans.

Exact existing policy tests still pass. New regressions compare 20,000 mixed
readiness/gate/reparenting operations with the old selection model and 10,000
CPU samples/rate/tree changes with independent eager accounting. Another 20,000
heap mutations compare expiry/removal with an ordered model across the u64
boundary. A three-account workload inspects the same scheduler work in a
three-slot and a 50,000-slot arena; 10,000 future sleepers require only one root
inspection per idle dispatch. These host tests do not raise the OS task limit
or establish constant-time selection for a large *active* sibling set.

The x86 smoke suite now includes a periodic userspace task beside busy and
yielding peers, then with an intentionally inadequate 1,000-us CPU ceiling.
Sixteen jobs use absolute releases eight nominal 100-Hz ticks apart and consume
at least 1,000 accounted microseconds each. Samples are buffered before serial
output. In this run both uncapped modes started and finished in the release
tick (coarse observations, not zero elapsed time); the capped mode reached 61
ticks of start lateness and 73 ticks of response, with 15 of 16 jobs finishing
past the next period. Peer progress, cap enforcement and complete reclamation
passed. This is an overload/timing baseline, not guaranteed service admission.
Serial records are in `build/boot-harness-rs/smoke/serial.log` (`CPU_TIMING`).

Read the [timing audit](source-rs/kernel/core/scheduler/timing.md) before extending
this work. Trap dispatch still scans IPC/readiness/device/task waiters and
pending work. Boot admission, whole page operations and descendant reclamation
still run with IRQs masked; the heap and polling diagnostic/device paths also
need bounded-time review. The pinned session's exclusive borrow forbids simply
enabling interrupts inside those operations. Next: event-owner waiter indexes,
explicit consistent suspension points for long operations, then a separately
admitted restricted timing profile. Ceilings still are not reservations.
Compiler requests, trusted evidence and runtime grants remain distinct; no
Omega certificate ABI, priority policy or real-time guarantee was invented.

Verification: workspace host tests; all 26 scheduler tests;
default UEFI and smoke-feature clippy, supervision-userspace clippy; Rust format,
layout and whitespace checks; x86 release smoke including the new periodic lab;
ARM64 release portability proof. Logs: `build/scheduler-host-tests.log`,
`build/scheduler-policy-tests.log`, `build/scheduler-smoke.log`,
`build/scheduler-arm64.log`.

The nested desktop pointer stress run passed its fresh boot, deepest saved
Notes, dirty-close protection and two subtree close/reopen cycles, but the cold
boot refused a clean outer close during the second cycle:
`desktop/20261020T185301025767Z`. The log records `matrix-close-refused` and
recursive `matrix-loss-uncertain` with TIMED_OUT, without kernel panic or matrix
exit. Child control LOSS/CLOSE still use `time::after(100)` per hop; an uncertain
answer correctly cannot authorize discarding the subtree. This is an outstanding
timing-sensitive close failure, not a claimed passing full two-boot stress run.

An unchanged revised-kernel cold replay on a copy of that exact saved fixture
passed all three levels, both teardown/reopen cycles and isolated outer Notes:
`desktop/scheduler-cold-recheck`. The same copied saved fixture also passed with
the preserved pre-change EFI from `desktop/20261020T173847824501Z`, at
`desktop/scheduler-baseline-cold`. Neither replay reproduced the refusal, so its
relationship to this scheduler change is not established.
Keep this regression visible; do not silently increase
control deadlines, treat a timeout as permission to discard, or weaken the
fixture to hide it. All desktop checks use disposable/copied media; the owner's
interactive disk has not been modified.

## Event-targeted wait completion (2026-10-20)

Baseline: `46cd674a`. Start at
[waits.rs](source-rs/kernel/core/user_tasks/execution/waits/waits.rs), then its
[routing guide](source-rs/kernel/core/user_tasks/execution/waits/README.md).
Syscalls and interrupts no longer probe every task's IPC, readiness and child
waits. IPC emits coalesced receiver hints for message delivery, revocation,
peer/grant closure, escrow discard, receive-right movement and boot rebinding.
Device policy targets installed owners; reclamation targets subscribers to the
changed child launch. A preallocated indexed heap selects only due wait timers.
Validated physical device routes and the keyboard owner are cached at startup.

Hints carry no authority or result. The existing syscall owners still validate
live grants and tickets, check copyout, and prefer queued data or terminal
results over expiry. Whole-set readiness validation is unchanged. Cancellation
removes subscriptions and timers before slot reuse; replacement cannot inherit
an old child wait or device hint. Registration always follows a current-state
check, avoiding lost readiness when an earlier hint was drained.

This is receiver-level indexing, not one subscription per endpoint. An unrelated
endpoint of the same task may cause a harmless bounded recheck. Bitmaps retain
the current 31-task arena limit; no OS capacity increase is claimed. Wake routing
allocates no storage, but simultaneous completions still require proportional
work. No kernel ABI, platform policy, distribution behavior, priority scheme or
reservation contract changed.

Verification:

- Workspace host tests pass; all 126 core tests pass after adding event-route,
  device replacement, cancellation/reuse, wraparound and populated-waiter
  regressions. IPC coverage includes byte/memory sends, moved receive rights,
  escrow closure and boot-pair replacement.
- Default UEFI and smoke-feature clippy pass; source layout, Rust formatting and
  whitespace checks pass. x86 release smoke and ARM64 portability proof pass.
  Logs: `build/waiter-host-tests.log`, `build/waiter-core-tests.log`,
  `build/waiter-clippy.log`, `build/waiter-smoke-clippy.log`,
  `build/waiter-smoke.log`, `build/waiter-arm64.log`.
- Three nested desktops under continuous pointer motion pass on both fresh and
  cold boots, including delayed/mismatched polling, deepest saved Notes,
  dirty-close protection, two subtree teardown/reopen cycles per boot and
  isolation: `desktop/20261020T191448080068Z`.
  This run did not reproduce the prior intermittent clean-close LOSS timeout;
  its cause remains unresolved, and the prior section's evidence still applies.
  Interactive saved media was not modified.

Next: index deferred admission/reclamation work, then introduce explicit
consistent suspension points for long IRQ-masked operations. Follow the
[timing audit](source-rs/kernel/core/scheduler/timing.md). Remaining scans,
grant lookup, heap behavior and device polling still preclude a bounded latency
claim. The exclusive session borrow still forbids simply enabling interrupts
inside ongoing operations. Owner decisions are not blocking these mechanics;
a separately admitted timing profile and its policy remain future work.

## Indexed deferred work and incremental allocation (2026-10-20)

Baseline: `6f73a85c`. Follow the new
[deferred-work entrance](source-rs/kernel/core/user_tasks/execution/deferred/deferred.rs)
and its [suspension/lifetime guide](source-rs/kernel/core/user_tasks/execution/deferred/README.md).
Trap-side pending detection no longer scans task/launch arenas or retired-memory
words. Accepted work populates fixed sets for admission, memory, cancellation,
retired tasks and loan return. Admission and memory steps rotate across pending
slots. Teardown clears queued calls/admissions before reuse, drains newly added
descendant cancellations, then returns loans only after physical reclamation.
Cached child-to-launch routes survive lending and dynamic reservation reuse.
Executable-reference and IPC escrow retirement visit marked entries rather than
every entry; their bounded bitmap-word iteration remains.

A boot step reclaims, admits at most one child and advances at most one memory
caller. After it returns, with no mutable session borrow or allocator lock held,
an explicit interrupt window and scheduler yield may run peers even while work
remains. A peer's next trap returns to boot for progress. An asynchronous boot
IRQ cannot bypass pending cleanup. This preserves deferred work on the boot
stack and avoids an artificial timer-tick delay when every caller is blocked.

Private allocation now reserves quota/funding once, acquires and zeros at most
eight pages per step, and retains unpublished progress in session state. It
returns no handle and exposes no mapping until all pages are ready. Each batch
is charged to its requester. Cancellation/failure refunds unbacked reservations
and allocated frames exactly once, before account reuse. Conservation checks
include reserved-but-unbacked pages. Final mapping and rollback remain whole
operations. The allocation batch size is a work allowance, not a latency promise.

Review caught a race newly possible across queued admissions: a consenting peer
can exit while another boot step runs. Admission now rechecks its original
consent, returning `PEER_CLOSED` before allocating/publishing a child instead of
reaching the live-peer assertion in IPC boot-pair installation. A replacement
task does not inherit that consent. Owner death removes the queued request.

Verification:

- Workspace host tests pass; all 130 core tests pass after the final peer-consent
  regression. Pending-set tests cover coalescing, rotating requeue, cancellation,
  reuse, newly queued descendants and boot/user routing.
- The real-frame rollback fixture fails maximum-size allocation at every frame
  boundary and cancels it at all 15 partial eight-page boundaries. Every case
  checks exact funding/frame recovery; partial batches expose no writable map.
  x86 release smoke passes, including IPC deadlines, cancellation/restarts,
  hierarchical CPU contention, memory faults and final heap/frame reclamation.
- ARM64 release portability passes; its private-memory peers now allocate full
  128-page regions through multiple steps, check zeroing/private data across
  preemption, and reclaim them after peer faults. ARM64 also runs the partial
  allocation rollback fixture.
- Default UEFI, smoke-feature and ARM64 portability-program clippy pass; layout,
  Rust formatting and whitespace checks pass. Logs are `build/deferred-*.log`.
- Three nested desktops under continuous pointer motion pass on fresh and cold
  boots, including deepest saved Notes, protected close, repeated teardown/reopen
  and isolation: `desktop/20261020T193010071854Z`. This desktop run preceded only
  the final queued-peer-consent check and test-only ARM64 expansion; the final
  kernel also passed x86 smoke. The earlier intermittent LOSS timeout did not
  recur and remains unresolved. Interactive saved media was not modified.

Next: make ELF construction and physical subtree reclamation resumable, with
private progress and cancellation state before further suspension boundaries.
Map/seal/unmap, allocation rollback, heap behavior and large event batches also
remain non-preemptible. The [timing audit](source-rs/kernel/core/scheduler/timing.md)
tracks these limits. This work does not change application ABI, platform policy,
distribution UI, CPU priorities or reservation semantics, and makes no measured
desktop-speedup or real-time guarantee. No owner decision blocks the next
mechanical steps.


## Resumable executable admission (2026-10-20)

Baseline: `8555aed8`. Live launch now follows
[lifecycle/spawn.rs](source-rs/kernel/core/user_tasks/lifecycle/spawn.rs): validate
current authority, advance a private draft, then prepare and commit its grants.
[Image decoding](source-rs/kernel/core/user_tasks/admission/image.rs) translates
today's ELF/probe input into segment descriptions. The resumable page engine,
funding, cancellation and publication do not depend on ELF as the future format.
No new executable or application ABI is established.

The first step creates an inactive root. Subsequent steps initialize/map at most
eight image or stack pages, then prepare runtime tables and any framebuffer in
separate phases. Launch state retains the immutable source and private draft;
no runnable context, task root or transferred endpoint appears before the final
serialized commit. Every step rechecks original peer consent and initial rights.
Delegation/grant plans and epochs are prepared after the last yield. Frame use
is charged incrementally to the child, CPU work to the requesting owner.

Failure or owner/peer death refunds all draft frames. Returning a dead owner's
launch first aborts any draft; no surviving lender means its executable-source
reference is also released. Conservation includes unfinished roots. Initial boot
admission remains synchronous. Decoding, kernel-root construction, runtime table
reservation, framebuffer mapping, final commit and rollback are still whole
operations; the batch size is a work allowance, not a latency guarantee.

Verification and findings:

- Workspace host tests pass, including 131 core tests. The new cursor regression
  covers partial pages, gaps, differing batch sizes and the stack without skips
  or duplicates.
- The real-frame admission fixture runs on x86 and ARM64. It fails at every frame
  allowance, withdraws at every stage boundary, checks BSS/permissions and proves
  that grants stay with the owner until commit. Production lifecycle cases cover
  owner/peer death, a queued message making an initial receive right unmovable,
  and reclamation of an abandoned object-backed source.
- Large by-value fixture sessions overflowed boot-stack headroom during development;
  the retained fixture session is now heap-owned and scenarios have separate stack
  frames. The fixture also avoids assuming syscall result registers match x86.
  ARM64 exception entry still saves onto the interrupted stack before switching
  stacks: audit stack-exhaustion reporting/emergency entry separately. The observed
  ARM fault report was delayed and did not identify the initial overflowing frame.
- The nested desktop harness incorrectly required dirty state after cold-open
  typing followed by deletion restored the saved document. Its screenshot showed
  the correct text and clean background; the expectation now distinguishes fresh
  unsaved content from cold-open confirmed content. Failure evidence:
  `desktop/20261020T200328028156Z`.

- Release x86 smoke passes with `--timeout 90`, including IPC, contention,
  cancellation/restart, display protection and final heap/frame reclamation.
  The first complete-suite attempt exceeded the default 30-second harness budget
  while still progressing; this does not change any guest protocol deadline.
- Release ARM64 portability and the executable-authority integration suite pass.
  Default x86 UEFI, smoke-feature x86 UEFI and smoke-feature ARM64 UEFI clippy pass,
  along with layout, formatting and whitespace checks.
- Logs: `build/admission-host-tests.log`, `build/admission-clippy.log`,
  `build/admission-smoke-clippy.log`, `build/admission-smoke.log`,
  `build/admission-arm64.log`, `build/admission-authority.log`.

- Fresh and cold desktop boots pass with three nested desktops, continuous pointer
  motion and deliberately delayed/mismatched interaction replies. Deepest Notes,
  clean/dirty close, persistence, isolation and two teardown/reopen cycles per boot
  pass: `desktop/20261020T201043556840Z`, `build/admission-desktop.log`.
  Interactive saved media was not modified.

The previously recorded intermittent clean-close LOSS timeout remains unresolved;
this milestone does not weaken its assertions or change protocol deadlines.

Next: bound subtree reclamation and mapping/rollback phases, preserving authority
withdrawal before physical reuse and ancestor funding return. Measure remaining
uninterrupted phases under competing work before making timing claims. Follow the
[timing audit](source-rs/kernel/core/scheduler/timing.md). No owner decision blocks
that mechanical work; guaranteed-service admission remains a separate design step.


## Incremental subtree reclamation (2026-10-20)

Baseline: `1428451e`. Start at
[lifecycle.rs](source-rs/kernel/core/user_tasks/lifecycle/lifecycle.rs), which
withdraws canceled authority, advances physical cleanup, then checks conservation.
[Reclamation](source-rs/kernel/core/user_tasks/lifecycle/reclamation.rs) separates
stopping tasks from collecting their slots and publishing outcomes.

Cancellation still stops the complete affected subtree and closes endpoints,
waits and device routes before the first yield. Physical retirement then rotates
among dead tasks. Alias cleanup scans at most 32 reader/region entries and cleans
at most one page object per step. After aliases disappear, each step returns at
most eight private image/stack/table frames. Abandoned admission drafts use the
same eight-frame allowance. Funding refunds track the remaining frame ledgers;
retirement CPU work is charged to the dead task, draft cleanup to its requester.
Accepted live readers retain backing after producer death.

Dead slots remain Exited until the entire cleanup wave finishes. Final metadata
collection publishes child outcomes, returns launch loans and completes blocked
cancellation callers. Admission waits behind cleanup and pending executable/IPC
reference refunds; existing users and their IPC/memory work can continue between
steps. Preventing new admissions also prevents a stream of new tasks from keeping
one cleanup wave alive indefinitely. The boot loop now drains pending cleanup
even after the last live task exits. No application ABI or distribution policy
changed.

Remaining large operations: complete authority withdrawal over the current
31-slot arena, final metadata collection, conservation assertions, individual
page-object unmap/free, abandoned private-allocation rollback, pending-reference
settlement and the previously recorded loader phases. This is a work allowance,
not a certified time bound. New launches can wait behind unrelated cleanup;
per-subtree collection dependencies are a future refinement if measurements
justify the extra state.

Verification:

- Workspace host tests pass, including 132 core tests. Pending-set coverage now
  checks that physical retirement, collection and abandoned drafts all hold the
  admission gate until their work is removed.
- The real-frame fixture cancels a committed parent/child and checks every step:
  at most eight private frames returned, endpoints withdrawn before the first
  pause, unaffected peer still Ready, both slots unavailable and child outcome
  unpublished until collection. Final frame/funding refunds are exact. Existing
  abandoned-source and interrupted-admission tests now drain multiple steps.
- Release x86 smoke and ARM64 portability pass with staged teardown, including
  live shared readers, peer faults, repeated cancellation/restarts and complete
  heap/frame reclamation. Executable-authority and IPC channel/startup-bundle
  integration suites pass. Default x86, smoke-feature x86 and smoke-feature ARM64
  UEFI clippy, layout, formatting and whitespace checks pass.
- Fresh and cold three-level desktop runs pass with continuous pointer motion,
  delayed/mismatched interaction replies, Notes persistence, clean/dirty close,
  isolation and two subtree teardown/reopen cycles per boot:
  `desktop/20261020T211451406375Z`. This preceded only the final gate holding
  admission until pending executable/IPC reference refunds settle; the final
  kernel is checked by the x86/ARM64 and executable-authority suites.
  Interactive saved media was not modified. The earlier intermittent clean-close
  LOSS timeout did not recur; its cause remains unresolved.
- Logs: `build/reclamation-host.log`, `build/reclamation-clippy.log`,
  `build/reclamation-smoke-clippy.log`, `build/reclamation-arm64-clippy.log`,
  `build/reclamation-smoke.log`, `build/reclamation-arm64.log`,
  `build/reclamation-desktop.log`, `build/reclamation-authority.log`,
  `build/reclamation-ipc.log`.

Next: measure the largest remaining masked phases under competing work, then
split single-object unmap/free and pending-reference retirement where those
measurements warrant it. Retain the ARM64 stack-exhaustion entry/reporting audit
from the previous milestone. Guaranteed-service admission remains separate from
these mechanisms; no new owner decision is needed for this cleanup work.

## Kernel-phase observations and single-pass page edits (2026-10-20)

Baseline: `09c1e774`. [Execution timing](source-rs/kernel/core/user_tasks/execution/timing/timing.rs)
buffers selected uninterrupted Rust-body spans, including nested trap/deferred
work, admission construction, reclamation and individual memory operations.
Only `memory-lab` builds collect them; normal F5 builds have no additional sample
storage or clock reads. Reports print after the session stops, outside measured
work. The [timing audit](source-rs/kernel/core/scheduler/timing.md) describes the
sampling boundaries and [report tool](tools/boot-harness-rs/timing_report.py).

The periodic probe now has two additional competitors: repeated one-page and
maximum-size allocations, freeze, unmap, remap and release. Both leave four owned
objects at exit, two unmapped, so actual task cleanup is measured too. The first
development fixture tried to keep four maximum-size objects mapped, exceeding
the separate address-space mapping ledger limit. That fixture was corrected;
no production capacity was enlarged to accommodate the test.

Measurements plus inspection identified repeated ledger searches during seal
and unmap. [Region edits](source-rs/kernel/core/user_tasks/address_space/regions.rs)
now traverse the ledger once; unmap compacts it while removing translations.
Both remain atomic. There is no new suspended mapping state, handle behavior,
authority rule or scheduling policy. The real-frame fixture checks every removed
page, every retained owner page, read-only state and surviving writable stack,
including table-boundary and last-slot aliases on both ISAs.

The complete baseline and final x86 smoke runs pass. Host load makes their elapsed
maxima and averages too variable for a reliable speedup claim (including spikes
in tiny operations). The structural improvement is removal of repeated scans;
this does not establish a desktop latency gain or a real-time bound. The combined
decode/root-creation phase repeatedly registers millisecond-scale work and is
the next useful place to narrow instrumentation before adding more state machines.
Initial boot admission and assembly entry/exit are outside the runtime samples.

Verification:

- Workspace host tests pass, including 132 core tests. Default x86 UEFI and
  smoke-feature x86/ARM64 clippy pass. Release x86 smoke and ARM64 portability
  pass, including real page tables, permissions, shared readers, failure rollback,
  subtree cleanup and full heap/frame refunds. Layout, formatting and whitespace pass.
- Fresh and cold three-level desktop runs pass with pointer stress, delayed/
  mismatched interaction replies, deepest Notes persistence, protected close,
  isolation and two teardown/reopen cycles per boot:
  `desktop/20261020T224834092056Z`, `build/phase-desktop.log`. This exercises the
  ordinary build with timing collection disabled. Interactive saved media was
  untouched. The earlier intermittent LOSS timeout did not recur.
- Logs: `build/phase-host-final.log`, `build/phase-default.log`,
  `build/phase-clippy-final.log`, `build/phase-arm64-clippy.log`,
  `build/phase-smoke-baseline.log`, `build/phase-smoke-final.log`,
  `build/phase-arm64.log`. Preserved observations:
  `build/phase-baseline-serial.log`, `build/phase-final-serial.log`.

Next: separate executable decoding from root/table construction costs, then
bound whichever phase warrants it. Pending-reference bursts and single-object
cleanup remain atomic. Keep the ARM64 exhausted-stack entry/reporting audit and
the intermittent clean-close LOSS timeout open. Guaranteed-service admission is
still separate; these mechanical changes need no owner decision.

## Resumable supervisor mappings during admission (2026-10-20)

Baseline: `fab5207d`. Separating `Decode` and `Root` observations identified x86
supervisor mapping construction as the substantial part of the old `DecodeRoot`
phase. In the baseline's 32-restart fixture, decoding totaled 1,363 us and root
construction 214,587 us. These are QEMU elapsed observations, affected by host
scheduling and calibration, not native cost bounds.

[Construction](source-rs/kernel/core/user_tasks/admission/construction.rs) now
allocates a private root, then prepares its supervisor mappings before loading
user pages. Core supplies a 64-leaf work allowance; the
[x86 backend](source-rs/kernel/arch/isa/x86_64/user/memory/creation.rs) owns the
boot-root address, kernel image/heap/stack ranges and cursor. Each step preserves
the original supervisor permissions. No mapper or mutable page-table borrow
survives the return to boot. The root accessor refuses an unfinished supervisor
tree. ARM64 retains its existing permanent supervisor-branch borrow and completes
the same preparation interface immediately. Initial boot admission runs this
engine synchronously; ordinary child launches may yield between batches.

All partial table allocations stay in the existing core frame ledger. Funding is
charged after each step and refunded on refusal or owner/peer death. Source and
initial-grant custody, revalidation and final publication remain unchanged.
This is a smaller uninterrupted work unit, not less total work: in the final
32-restart sample, bare-root allocation totaled 747 us and mapping preparation
258,996 us spread across 2,112 steps. Both binaries include lab programs in their
kernel image; normal F5 has a different mapping footprint. Admission can take
longer under contention because peers can run between those steps.

The longer startup exposed a test-interval error in the CPU-turn fixture: early
children spun while later siblings were still being built, but the assertion
compared their lifetime preemption counts against an outside peer that began
later. Workers now park on their existing IPC ports until all admissions finish
and the owner starts measurement. The fairness tolerance is unchanged. Final
counts were outside/subtree 64/64 with one spinner and 65/65 with three. The
admission rollback fixture's old fixed 64-step stall guard also had to account
for the embedded kernel image; no production capacity or deadline was enlarged.

Verification:

- Workspace host tests pass, including the new x86 cursor regression over empty
  ranges, gaps, high addresses and differing batch sizes, plus 132 core tests.
- Release x86 smoke and ARM64 portability pass. Physical admission coverage
  fails every frame allowance, cancels at every stage (now including supervisor
  batches), checks unpublished authority and exact refunds, and checks owner/
  peer loss and retained executable-source reclamation. It verifies multiple
  x86 supervisor steps and one ARM64 preparation step.
- Executable-authority integration passes: recursive funded pools, executable
  capabilities, failed loads, stale grants and surviving siblings. Default x86,
  smoke x86/ARM64 and supervision-guest clippy pass. Layout, formatting and
  whitespace checks pass.
- Fresh and cold three-level desktop runs pass with pointer stress, deepest
  Notes persistence, protected close, isolation and two teardown/reopen cycles
  per boot: `desktop/20261020T232137338052Z`, `build/root-desktop.log`. This
  exercises the ordinary build with timing collection disabled. Interactive
  saved media was untouched. The intermittent clean-close LOSS timeout did not
  recur; delayed interaction replies were not enabled in this run.
- Logs: `build/root-host.log`, `build/root-baseline.log`,
  `build/root-baseline-serial.log`, `build/root-smoke-final.log`,
  `build/root-final-serial.log`, `build/root-arm64.log`,
  `build/root-authority.log`, `build/root-default.log`,
  `build/root-clippy-final.log`, `build/root-arm64-clippy.log`,
  `build/root-guest-clippy.log`. The initial test-interval failure is retained in
  `build/root-smoke.log` and `build/root-pre-barrier-serial.log`.

Next: use the now-separated phases to choose among runtime-table preparation,
framebuffer mapping, rollback and reference-retirement bursts. Do not infer a
latency guarantee from the work allowance. The ARM64 exhausted-stack entry audit,
intermittent clean-close LOSS timeout and guaranteed-service admission remain
open; no owner decision was needed for this mechanical step.

## Staged framebuffer admission and retirement observations (2026-10-20)

Baseline: `929db9c8`. The next audit found a sampling problem: `RegionTables` and
`Framebuffer` counted admissions that requested neither mapping. Those no-op
samples no longer dilute the reports. The real-frame admission fixture now
reports a complete private construction and, on x86, receives the actual reserved
GOP aperture. Initial boot admission remains outside these runtime samples.

[Construction](source-rs/kernel/core/user_tasks/admission/construction.rs) now
advances framebuffer mapping by at most 64 device leaves per step. Core's
[framebuffer coordinator](source-rs/kernel/core/user_tasks/address_space/framebuffer.rs)
retains the byte offset and tracks table allocations; the
[x86 device backend](source-rs/kernel/arch/isa/x86_64/user/memory/device.rs)
validates each slice and preserves RW/NX, guards and uncached PAT policy. Device
backing never becomes owned RAM or accessible through ordinary checked-copy
helpers. The draft stays private and unrunnable until all mappings and the final
authority commit complete. Every step retains the existing owner/peer and initial
endpoint revalidation, CPU charging and physical funding. Partial failure or
withdrawal refunds all owned tables, never the device aperture.

The normal 1024x768 GOP aperture spans 768 pages and now takes 12 mapping steps;
the contract's maximum 16 MiB aperture takes at most 64. Initial boot uses the
same engine synchronously. ARM64 keeps its explicit unsupported framebuffer
result; no display backend, ABI, deadline or capacity changed. The framebuffer
coordinator is a named sibling of image construction and page-object mappings;
`mod.rs` remains plumbing.

A new [reference-retirement fixture](source-rs/kernel/core/user_tasks/memory/references_test.rs)
creates 70 real objects across bitmap words: 65 one-page objects and five
128-page objects, each held by two kernel references. It drops those references
through production settlement, verifies exact backing/funding refunds and keeps
one sealed owner mapping readable. The baseline settled the burst in 30 us;
the final x86 sample was 49 us and ARM64 155 us. Settlement remains atomic: these
observations do not justify prioritizing another state machine there yet, and
this is not a full-arena worst case or a competing-workload measurement.

The new complete-construction sample observed runtime-table preparation at
140 us on x86 and 134 us on ARM64. x86 framebuffer preparation totaled 828 us
across 12 batches, with a 412 us maximum. Earlier successful whole-framebuffer
samples were in the hundreds of microseconds; batching does not remove work,
and host scheduling makes these unsuitable for a speedup or latency-bound claim.
The reason for this split is the explicit cap on uninterrupted leaf edits.

Verification:

- Workspace host tests, default x86 UEFI clippy and smoke-feature x86/ARM64 clippy
  pass. Layout, formatting and whitespace checks pass.
- Release x86 smoke passes. Real-frame admission cancels at every construction
  boundary, now including each framebuffer batch, and fails every physical frame
  allowance while checking unpublished context/IPC state and exact refunds.
  Display probes still pass NX/guard faults, checked-copy rejection, provider
  restart and every device-table allocation failure. ARM64 portability passes,
  including admission rollback and the new reference burst.
- Fresh and cold three-level desktop runs pass with pointer stress, protected
  close, deepest Notes persistence, isolation and two teardown/reopen cycles per
  boot: `desktop/20261020T234449530073Z`, `build/aperture-desktop.log`. These use
  ordinary builds with timing collection disabled. The interactive saved disk
  was untouched. The intermittent LOSS timeout did not recur; delayed interaction
  replies were not enabled in this run.
- Logs: `build/burst-baseline.log`, `build/burst-baseline-serial.log`,
  `build/aperture-smoke.log`, `build/aperture-final-serial.log`,
  `build/aperture-host.log`, `build/aperture-clippy.log`,
  `build/aperture-default.log`, `build/aperture-arm64-clippy.log`,
  `build/aperture-arm64.log`.

Next: runtime-table preparation still runs atomically; ARM64 currently walks
all leaves to reserve tables where one visit per leaf-table span should suffice.
Measure that simplification before adding a cursor. Whole error rollback,
single-object cleanup and reference/escrow bursts remain bounded only by their
capacity limits. Keep the ARM64 exhausted-stack entry/reporting audit and the
intermittent clean-close LOSS timeout open. Guaranteed-service admission remains
a separate design decision; this milestone required no owner decision.

## ARM64 runtime-table reservation by span (2026-10-20)

Baseline: `601edf25`. [ARM64 user mappings](source-rs/kernel/arch/isa/aarch64/user/mappings.rs)
now reserve one address per 2-MiB leaf-table span. Advancing to the next boundary
also handles a start inside a table. For the current 52 guarded mapping slots,
this reduces 6,708 walks to 14 while allocating the same tables and installing
no user leaves. No new cursor, ABI, capacity or scheduling policy is needed;
reservation remains atomic. Core retains physical custody and rollback.

The shared [real-frame fixture](source-rs/kernel/core/user_tasks/memory/rollback.rs)
now repeats reservation and checks that no frames were added, then maps/unmaps
every reserved slot with a maximum-size object. Runtime mapping prohibits table
allocation, so an omitted table fails directly. The test checks each page's
contents and read-only state, guard rejection, removal of every alias, and
preservation of the owner mapping and stack. Existing failure injection still
covers every admission allocation and exact frame/heap refunds.

The complete-construction ARM64 sample recorded runtime-table preparation at
121 us before and 50 us afterward. Other samples varied upward as well as down;
these host-sensitive observations are not a latency bound or a reliable speedup
ratio. The defensible improvement is removal of 6,694 redundant table walks.

Verification: release ARM64 baseline and final portability suites pass,
including the expanded mapping test and all 66 admission failure boundaries.
Release x86 smoke also passes the expanded shared fixture. Workspace host tests,
x86/ARM64 smoke-feature clippy, layout, formatting and whitespace checks pass.
Production changes are ARM64-only; the nested graphical desktop test was not
repeated, and interactive saved media was untouched. Logs:
`build/reserve-arm64-baseline.log`, `build/reserve-arm64-final.log`,
`build/reserve-host.log`, `build/reserve-arm64-clippy.log`,
`build/reserve-x86-smoke.log`, `build/reserve-x86-clippy.log`.

Next: prioritize the outstanding ARM64 exhausted-stack entry/reporting audit.
Runtime table reservation is now a small fixed number of walks at current
limits. Whole error rollback, single-object mapping/cleanup, reference/escrow
bursts and event bursts remain atomic; do not claim bounded-time execution.
The intermittent clean-close LOSS timeout and guaranteed-service admission stay
open. No owner question was needed for this simplification.

## ARM64 stack-safe exception entry and terminal reporting (2026-10-20)

Baseline: `1fe5af72`. The previous entrance saved its 800-byte context onto the
interrupted EL1 stack before switching to the permanent IRQ stack. A new probe
moved SP into the boot guard and faulted on an identified store; the baseline
timed out without a fault report (`build/stack-before.log`,
`build/stack-before-serial.log`). This reproduces the earlier diagnostic gap.

[Entry assembly](source-rs/kernel/arch/isa/aarch64/context/entry.S) now saves
normal contexts directly into the reserved top of the permanent IRQ stack.
A transient `TPIDR_EL1` scratch value preserves x0 while x1 and the original SP
are saved there; the register is reserved by this backend and cleared before
Rust dispatch. No store depends on the interrupted kernel stack. Register layout,
EL0 SP handling, shared scheduling and return-root selection remain unchanged.

Fatal EL1 synchronous faults take a separate assembly entrance before any normal
context save. It captures the original ESR/FAR/ELR/SP, switches to a distinct
permanent fault stack, and calls the existing terminal reporter. FIQ/SError also
use the terminal entrance. A second fatal fault parks without recursively
restarting reporting. Returning BRK/SVC/IRQ paths retain ordinary dispatch, and
EL0 faults still go through core's process-containment path. This remains a
single-CPU EL1h profile with masked dispatch and no nested resumable callbacks.
Kernel exhaustion is terminal; emergency-stack/mapping corruption is not recovered.

The new [boot probes](source-rs/kernel/boot/uefi/smoke/stack.rs) require the exact
faulting instruction, guard address, original SP and reporting-stack range for
both boot-stack and interrupt-stack exhaustion. A positive probe gives BRK, SVC
and a real timer IRQ only 16 bytes of interrupted-stack headroom, checks x0/x1
preservation, and returns to the original caller stack. Run with the ARM64
harness's `--fault stack-exhaustion`, `--fault irq-stack-exhaustion`, or
`--fault stack-entry`; these probes are absent from normal boots.

Verification:

- All three targeted probes pass in release and debug builds. The seven existing
  ARM64 guard/code/data/NX probes pass in release. The full release portability
  suite passes, including EL1/EL0 register preservation, timer preemption, IPC,
  user-fault containment, admission rollback and full reclamation.
- Workspace host tests, ARM64 smoke/probe clippy, default x86 UEFI clippy, layout,
  formatting and whitespace checks pass. Production code changes are ARM64-only;
  no graphical desktop test or x86 QEMU rerun was needed. Saved desktop media
  was untouched.
- Full debug portability does NOT pass: after the trusted-task tests it reports
  a read translation fault in `__chkstk` before shared-memory userspace work.
  Diagnostic: vector 37, ESR `0x96000007`, PC `0x43b7d31c`, FAR `0x1000000b00`,
  original SP `0x1000006b00`. The guard access occurs during stack probing before
  SP itself moves into the guard. The original instruction was resolved against
  that build's PDB/disassembly; the exact oversized caller remains to be located.
  This is now an immediate terminal report, not a delayed secondary entry fault.
- Logs: `build/stack-exhaustion.log`, `build/stack-irq-stack-exhaustion.log`,
  `build/stack-stack-entry.log`, `build/stack-debug-*.log`,
  `build/stack-arm64-release.log`, `build/stack-arm64-debug.log`,
  `build/stack-debug-overflow-serial.log`, `build/stack-host.log`,
  `build/stack-clippy.log`, `build/stack-probes-clippy.log`,
  `build/stack-x86-default.log`. Other protection logs are `build/stack-*.log`.

Next: trace and reduce debug userspace-session setup's stack pressure (inspect
large by-value Session/result temporaries) without widening stacks or discarding
guards. The exhausted-stack entrance/reporting gap is addressed; the newly
localized debug boot failure is separate. Keep the intermittent clean-close
LOSS timeout open. Atomic cleanup/event bursts and guaranteed-service admission
remain future work; no owner decision was needed here.


## Stack-safe, fallible userspace session startup (2026-10-20)

Baseline: `7331c086`. Full ARM64 debug portability now passes. The previous
`__chkstk` guard read came from stacked startup frames carrying inline session
and IPC startup-handle tables. The old debug userspace runner reserved 28,000
bytes and preparation reserved another 38,912 bytes, before their callees and
boot callers. The stack probe faulted before entering the oversized frame.

[Preparation](source-rs/kernel/core/user_tasks/startup/startup.rs) now returns
an owned heap session. [Arena construction](source-rs/kernel/core/user_tasks/startup/arena.rs)
allocates fallibly and initializes fields directly, without a whole-session
stack temporary. All fallible work precedes raw initialization; no partially
initialized session is published. The owner remains at a stable mapped address
until execution clears the switch handler and `ACTIVE`, then ordinary drop
reclaims its storage. The admission fixture uses that same owner, removing its
extra box/copy. This preserves the existing raw-pointer suspension obligations.

[IPC startup](source-rs/kernel/core/ipc/bootstrap.rs) reserves its initial-handle
rows for the configured task slots, including dormant children, once on the
heap. Rows never grow during a trap. Its allocation refusal now reaches callers
as `OutOfHeap`, not `InvalidEndpoints`. Scheduler accounts/deadlines, waiter
storage and lab timing samples also have fallible constructors used by userspace
startup. The trusted-kernel-task session still uses the existing infallible
`Scheduler::new` wrapper; this is not a whole-kernel allocation-failure audit.

Disassembly of the final ARM64 debug build shows preparation's probed allocation
at 6,432 bytes; the userspace runner no longer needs a stack probe. Other
constructors still have sizable debug frames. This is not a global stack-use
bound. Boot/task stack capacities and guards are unchanged, and the fatal-stack
entry/reporting mechanism from the previous milestone remains intact.

The new shared [startup fixture](source-rs/kernel/boot/uefi/lab/startup.rs)
exhausts the real kernel heap and releases it in 4-KiB increments. Every refusal
must return `OutOfHeap` with exact heap/frame recovery. Once startup fits, a zero
physical-frame allowance must refuse image admission, again refunding everything.
It reaches 73 heap refusals on both ISAs before that boundary. This samples
available capacities rather than injecting a fault at every allocation call.
Subsequent ordinary sessions still boot and reclaim successfully. A host test
also verifies that impossible scheduler capacity returns an error.

Verification:

- Full ARM64 debug and release portability pass, including the new heap-pressure
  fixture, private-process isolation, shared-object lifetime and admission rollback
  (78 frame boundaries in debug, 66 in release).
- Full x86 release smoke passes, including the same heap-pressure fixture,
  supervision, device/display recovery and exact memory reclamation.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. ARM64 kernel-only smoke excludes the
  userspace fixture because that build omits the required image metadata.
- Nested desktop regression covers fresh/cold boot, three recursive desktops,
  pointer stress, draft-protected close, confined persistence and repeated subtree
  reclamation. Disposable images only; interactive F5 saved media was untouched.
- Logs: `build/session-arm64-debug.log`, `build/session-arm64-release.log`,
  `build/session-x86-smoke.log`, `build/session-host.log`,
  `build/session-x86-clippy.log`, `build/session-x86-smoke-clippy.log`,
  `build/session-arm64-clippy.log`, `build/session-desktop.log`,
  `build/session-desktop-final.log`, `build/session-debug-stack-frames.txt`.

Next: both the exhausted-stack reporting gap and the resulting debug startup
failure are resolved. Return to the measured atomic cleanup/event bursts,
starting with rollback and single-object mapping/cleanup before adding more
cursors. Keep the intermittent clean-close LOSS timeout open; no failure in this
milestone proves that issue fixed. Guaranteed-service admission remains a
separate owner-level design choice. No new owner decision was needed here.


## Bounded failed-launch rollback (2026-10-20)

Baseline: `e5a13906`. A live process's failed child launch previously released
its entire draft synchronously, including errors after image/table construction
or final authority validation. It now retains the first refusal in the launch
account and uses later admission turns to return at most eight private frames
per visit, through the same frame-release engine as abandoned/dead-task cleanup.
[Spawn](source-rs/kernel/core/user_tasks/lifecycle/spawn.rs) shows the choice
between advancing construction and advancing rollback. No additional cleanup
cursor or ABI is introduced.

The parent remains blocked, the launch remains busy and its child stays
unpublished until all private frames are refunded. Then the original error is
returned and the launch becomes retryable. Initial rights/messages remain with
the parent throughout refusal. Each cleanup step charges the requester and
refunds the child's physical account. If the parent dies mid-rollback, close
clears its refusal, removes pending admission and transfers the partial draft
to ordinary abandonment before returning launch capacity/source custody.
Executable-object references remain pinned until the appropriate owner/loan
cleanup settles them. No partly retired root can resume or serve checked copies.

The [shared real-frame suite](source-rs/kernel/core/user_tasks/admission/rollback/rollback.rs)
now has a small sequencing entrance and named construction, refusal, authority
and fixture leaves. Existing sweeps still refuse every physical-frame allowance
and withdraw at every construction phase, including framebuffer batches on x86;
those tests assert the eight-frame cleanup allowance and exact refunds.
New production-lifecycle cases cover a failure at the last allocation, a busy
initial receive right after complete private construction, blocked-parent and
selectable-peer state between batches, no early launch reuse, successful retry,
and owner death after partial rollback. Object-backed executable tests cover
both interrupted construction and interrupted rollback with final source refund.

Verification:

- Full ARM64 debug/release portability and x86 release smoke pass the expanded
  suite, including all physical-allocation failure boundaries, authority checks,
  object lifetime, preemption and exact heap/frame reclamation.
- The release userspace admission profile passes recursive funded pools, load
  rollback, stale grants and sibling survival. The nested desktop regression
  covers three levels, pointer stress, protected close, confined persistence,
  repeated subtree reclamation and cold reopen on disposable images.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. Interactive saved media is untouched.
- Logs: `build/refusal-arm64-debug.log`, `build/refusal-arm64-release.log`,
  `build/refusal-x86-smoke.log`, `build/refusal-admission.log`,
  `build/refusal-desktop.log`, `build/refusal-host.log`,
  `build/refusal-x86-clippy.log`, `build/refusal-x86-smoke-clippy.log`,
  `build/refusal-arm64-clippy.log`.

This bounds private-draft frame returns per visit, not elapsed time. Heap drops
and bookkeeping remain atomic; pre-draft bare-root failure and synchronous
initial-boot construction still perform their small/whole rollback internally.
Runtime page-object mapping, unmapping/free, allocation rollback, pending
reference/escrow settlement and event bursts remain separate work. More turns
can lengthen the wait for a failed launch under contention; there is no faster
launch or hard-real-time claim. The intermittent clean-close LOSS timeout remains
open. No owner question was needed for this mechanism change.

## Bounded page-object backing retirement (2026-10-20)

Baseline: `2e10113a`. Runtime page-object release, allocation refusal/cancellation,
dead-task object cleanup, deferred executable references and discarded IPC
attachments now return at most eight physical frames per path visit. The
[memory entrance](source-rs/kernel/core/user_tasks/memory/memory.rs) selects each
owner; [retirement](source-rs/kernel/core/user_tasks/memory/retirement.rs) owns the
shared physical refund operation. This extends the existing boot-loop suspension
mechanism without a new syscall or scheduler policy.

Final release removes the alias first and keeps the caller blocked until its
backing refund completes. Death inherits a partly completed release. Allocation
failure refunds the unbacked reservation exactly once, then retires initialized
pages incrementally before returning the error. A retiring object retains its
metadata slot and whole-object logical quota until completion; physical funding
returns to its original sponsor batch by batch. Other users can continue memory
operations between visits, and live readers still pin shared backing.

[Executable references](source-rs/kernel/core/user_tasks/memory/references.rs)
rotate across pending objects, consuming surrendered reference counts once.
[IPC escrow](source-rs/kernel/core/user_tasks/memory/escrow.rs) drains one finite
discard snapshot before taking the next. Review found a slot-reuse hazard in
the incremental implementation: receiver teardown could release an unpublished
reader before its pending discard bit was consumed. Teardown now preserves
slots owned by either the current snapshot or the next IPC discard batch. A
consumed slot may safely be reused while its old object's backing finishes
retiring. The regression deliberately kills both receivers with both batches
pending, reuses the first slot, and checks that the new published right survives.

The [real-frame suite](source-rs/kernel/core/user_tasks/memory/retirement_tests/retirement_tests.rs)
also checks maximum-size release, blocked caller/ready peer state, competing
allocation, death after the first release batch and exact conservation after
each cleanup visit. Existing allocation sweeps refuse every frame allowance and
cancel at each partial batch, now checking the refund allowance in their test
wrappers. The 140-reference/70-object burst takes 130 bounded settlement visits
while retaining a live owner's sealed mapping. Host policy tests check retiring
slot/quota retention, stale authority and eventual safe reuse.

Verification:

- ARM64 debug/release portability and x86 release smoke pass, including the
  expanded real-frame suite and exact heap/frame reclamation.
- Release admission passes recursive funded pools, load rollback, stale grants
  and sibling survival. Nested desktop passes three levels, pointer stress,
  protected close, confined persistence, repeated reclamation and cold reopen.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. Interactive F5 saved media was untouched;
  automated desktop/admission runs used disposable images.
- Logs: `build/object-host.log`, `build/object-arm64-debug.log`,
  `build/object-arm64-release.log`, `build/object-x86-smoke.log`,
  `build/object-admission.log`, `build/object-desktop.log`,
  `build/object-x86-clippy.log`, `build/object-x86-smoke-clippy.log`,
  `build/object-arm64-clippy.log`.

The allowance is per cleanup path visit, not a total of eight frames across a
whole boot turn. Mapping removal still happens atomically before any backing
refund. Final map/seal/unmap, heap operations, metadata scans, collection,
authority withdrawal and event bursts remain uninterrupted. More cleanup visits
can increase completion latency under contention; these are structural work
bounds, not a desktop speedup or real-time guarantee.

Next: use the existing phase observations to choose between remaining mapping
work and bursts of event completions. Preserve mapping visibility and terminal/
deadline ordering when introducing further suspension points. The intermittent
clean-close LOSS timeout remains open; passing this regression does not diagnose
it. Guaranteed-service admission still needs its separate owner-level design.
No new owner decision was needed for this milestone.

## Batched wait completion (2026-10-20)

Baseline: `592aacd7`. [Wait completion](source-rs/kernel/core/user_tasks/execution/waits/waits.rs)
previously retried all affected receive, readiness and child waiters within one
invocation. It now selects at most eight candidates per visit and retains the
rest in the registry. A finite current wave preserves readiness-before-receive-
before-child order; later hints coalesce into the next wave so repeated low-slot
events cannot continually overtake older candidates. No allocation is introduced
on the wake path. Cancellation/completion clears both sets before slot reuse.

Pending candidates participate in the ordinary boot-work detection and interrupt
routing. The boot step continues a batch before yielding, without requiring a
fresh hardware event or runnable user. Syscall owners still validate the current
capability, destination, message/child state and deadline. Data or a terminal
error wins over timeout at the actual retry; no arrival timestamp or successful
result is cached in the registry. This preserves existing completion rules while
allowing completion to occur later under contention. No scheduling priority or
new application ABI is added.

Host tests cover a full 31-slot burst, ordering across kinds, draining without new
events, repeated low-slot hints and clearing both queues before a slot changes
wait kind. The [real-frame burst](source-rs/kernel/core/user_tasks/execution/waits/burst.rs)
parks 25 inactive roots and triggers 24 waiters. They complete over exactly three
visits, including two through the production deferred step with no new interrupt.
Assertions check queued data/terminal errors before expiry, readiness without
dequeue, a too-small receive preserving its message, empty-queue timeouts and
exact frame/heap recovery. This fixture does not execute guests; ordinary smoke
and desktop regressions cover real task suspension and execution.

Verification:

- ARM64 debug/release portability and x86 release smoke pass the expanded suite.
  The new buffered `WaitCompletion` phase records gathering plus one retry batch;
  it includes empty invocations and overlaps enclosing trap/boot observations.
- Nested desktop passes three recursive levels, pointer stress, protected close,
  confined persistence, repeated reclamation and cold reopen on disposable media.
- Workspace host tests and the final core tests pass, as do default x86 UEFI
  clippy, x86/ARM64 smoke clippy, layout, formatting and whitespace checks.
  Interactive F5 saved media was untouched.
- Logs: `build/wait-host.log`, `build/wait-core.log`,
  `build/wait-arm64-debug.log`, `build/wait-arm64-release.log`,
  `build/wait-x86-smoke.log`, `build/wait-desktop.log`,
  `build/wait-x86-clippy.log`, `build/wait-x86-smoke-clippy.log`,
  `build/wait-arm64-clippy.log`.

Remaining: eight candidates is a per-invocation allowance, not a whole-trap or
whole-boot-step limit. Subtree withdrawal can call wake repeatedly. Due waiter
timers still drain the at-most-31-entry heap atomically; scheduler sleepers,
physical keyboard and direct device wakes are separate. Grant lookup, mapping
edits and collection also retain atomic work. The timing audit names these gaps;
next work should measure aggregate closure/completion and separate ready-time
from dispatch-time observations before claiming latency improvements. The known
intermittent clean-close LOSS timeout remains undiagnosed. No owner decision was
needed for this mechanism change.

## Coalesced subtree withdrawal (2026-10-20)

Baseline: `6552c20a`. Cancellation previously called the ordinary task-close
entry for each descendant. Each close retried waiters, multiplying completion
batches inside the uninterrupted authority sweep. A descendant queued for later
cancellation could also receive/copy data before the sweep reached it.

[Lifecycle](source-rs/kernel/core/user_tasks/lifecycle/lifecycle.rs) now separates
standalone close from [task authority withdrawal](source-rs/kernel/core/user_tasks/lifecycle/withdrawal.rs).
[The cancellation sweep](source-rs/kernel/core/user_tasks/lifecycle/reclamation.rs)
stops and withdraws every affected descendant, including newly discovered
grandchildren, then runs one waiter batch for surviving observers. Slot, outcome,
loan and physical retirement rules are unchanged. A standalone close still has
its own completion batch; collection and the boot step may also retry waiters.
This reduces repeated work within the sweep without inserting a suspension point
while descendants retain canceled authority.

The shared [real-frame regression](source-rs/kernel/core/user_tasks/lifecycle/withdrawal_test.rs)
uses production admission and loan commits to construct a parent with eight
children. After the initial owner dies, it queues descendant messages and an
unrelated receive before the authority sweep. It checks that all nine descendants
are stopped and their rights/waits removed before the first pause, none consumes
or copies its queued data, and exactly one waiter-completion invocation occurs
for the whole sweep. The unrelated receiver completes successfully. Physical
cleanup then returns every frame and heap allocation while preserving that peer.

`Close` phase samples now expose standalone withdrawal plus completion. `Withdraw`
samples exclude empty cancellation visits and include the whole nonempty sweep
plus its single completion batch. Earlier averages included empty visits and
must not be compared as though this were an elapsed-time speedup. Both are
inclusive QEMU observations, with later physical retirement measured separately.

Verification:

- ARM64 debug/release portability and x86 release smoke pass, including the new
  subtree regression, existing cancellation counts, waiter burst and exact refunds.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. Interactive F5 media was untouched.
- Logs: `build/withdraw-host.log`, `build/withdraw-core.log`,
  `build/withdraw-arm64-debug.log`, `build/withdraw-arm64-release.log`,
  `build/withdraw-x86-smoke.log`, `build/withdraw-desktop.log`,
  `build/withdraw-x86-clippy.log`, `build/withdraw-x86-smoke-clippy.log`,
  `build/withdraw-arm64-clippy.log`.

Next: distinguish becoming runnable from actual CPU selection in the periodic
observations. The authority sweep, grant-table scans, deadline extraction and
mapping edits still run atomically; a whole trap/boot turn is not limited to one
waiter batch. The known intermittent clean-close LOSS timeout remains open.
No owner semantics or application ABI changed, and no real-time guarantee follows.

## Wait-to-selection observations (2026-10-20)

Baseline: `74ceb818`. Registered IPC receives, readiness waits and child waits
now record two separate observations in `memory-lab` builds: deadline lateness
at completion in ticks, and elapsed microseconds from completion/unblock to the
first selected return. The latter includes peer execution, boot work, CPU-ceiling
gates and host descheduling. It ends before assembly restores user context; it
does not timestamp the first user instruction or the original hardware event.

The [timing entrance](source-rs/kernel/core/user_tasks/execution/timing/timing.rs)
now lives in a named folder with separate phase definitions, wait observations
and host tests. Rust module plumbing contains no implementation. Storage is
reserved fallibly at session preparation; hooks allocate and print nothing.
Ordinary builds omit the extra storage, clocks and hooks. The ARM64 heap-pressure
fixture exercises 75 sampled startup refusals with the larger lab-only storage,
with exact reclamation; debug boot retains its existing guarded stack size.

`WAIT_DISPATCH` reports per-slot/per-kind completed, selected, canceled and still
pending populations. Cancellation clears unfinished timestamps before slot
reuse. Later returns cannot count a completion twice. Only selected observations
contribute to selection durations; only reached deadlines contribute to lateness.
Data/terminal-before-timeout rules are unchanged, so a reached deadline does not
necessarily mean the wait returned a timeout. No-deadline waits, immediate
syscall results and other wake mechanisms are not silently mixed into these
populations. The [contract and reading guide](source-rs/kernel/core/user_tasks/execution/timing/README.md)
records the full scope and units.

[The report tool](tools/boot-harness-rs/timing_report.py) displays these spans
separately from inclusive execution phases. `--verify-waits` checks periodic
observations against the actual number of registered blocked waits. Normal x86
smoke performs that check for all five periodic cases, requiring selected counts
to agree and no pending/canceled samples. It imposes no latency threshold; zero
blocked waits is valid when all deadlines are already expired at the call.
The capped workload can therefore have fewer blocked-wait samples than its 16
jobs. This is observable now rather than being mistaken for missing samples.

Verification:

- Workspace host tests pass, including a ready task held behind an actual
  scheduler CPU ceiling, first-selection accounting, cancellation/reuse, zero
  timestamps, deadline equality and wrapping ticks.
- ARM64 debug/release portability and x86 release smoke pass. All five x86
  periodic wait counts agree with selection reports. Report-tool tests reject
  missing/duplicate/inconsistent observations and preserve separate populations.
- Default x86 UEFI clippy, x86/ARM64 smoke clippy, source layout, formatting and
  whitespace checks pass. No ordinary desktop policy or saved F5 media changed;
  the nested desktop was not rerun for these lab-only hooks.
- Logs: `build/dispatch-host.log`, `build/dispatch-core.log`,
  `build/dispatch-arm64-debug.log`, `build/dispatch-arm64-release.log`,
  `build/dispatch-x86-smoke.log`, `build/dispatch-report.log`,
  `build/dispatch-x86-clippy.log`, `build/dispatch-x86-smoke-clippy.log`,
  `build/dispatch-arm64-clippy.log`. Python check:
  `python tools/boot-harness-rs/timing_report_test.py`.

Next: use these observations to select the next remaining atomic mapping,
authority or deadline operation. They do not justify changing CPU entitlement
or assigning privileged processes implicit priority. Guaranteed service remains
separate from ceilings. The intermittent clean-close LOSS timeout is still open;
these probes have not reproduced or diagnosed it. No owner decision was needed.

## Batched page-table permission/removal edits (2026-10-20)

Baseline: `d8450d09`. The page-object timing workloads exposed repeated hierarchy
walks inside atomic freeze/unmap operations. The
[core region coordinator](source-rs/kernel/core/user_tasks/address_space/regions.rs)
now sends the contiguous range to the selected ISA before updating its backing
ledger. [x86](source-rs/kernel/arch/isa/x86_64/user/memory/regions.rs) and
[ARM64](source-rs/kernel/arch/isa/aarch64/user/regions.rs) walk once per leaf table,
then edit only the requested entries. A current 128-page object needs one or two
walks instead of 128. The backends own table geometry/encoding; core retains
frame lifetime, capability publication and its single ledger pass.

There is no suspended cursor, allocation or new authority state. Both operations
remain atomic with an inactive root, pinned backing and masked interrupts. Leaf
writes still scale with object size; ledger work still scales with mapped pages.
The existing next-root synchronization remains required. This is less work in
an uninterrupted operation, not a real-time bound or an SMP design. Mapping new
leaves and its failure rollback remain unchanged.

The new x86 host test constructs tables through the x86 crate and translates the
result independently. It checks physical addresses, RO/NX permissions, complete
removal and untouched neighbors across PT, PD and PDPT boundaries. The shared
real-frame allocation fixture now seals temporary aliases at every guarded slot
alignment on both ISAs, then checks each removed page and surviving owner/stack
mapping. All backing is reclaimed. An initial fixture assertion incorrectly used
the single-page copy validator for an entire object; that assertion was corrected
to validate each page before the successful boot runs.

Verification:

- Workspace host tests and arch host clippy pass, including the new table test.
- x86 release smoke and ARM64 debug/release portability pass, including existing
  reader write-fault containment, allocation/refusal sweeps and exact reclamation.
- The report verifier accepts all five periodic wait populations. This run's
  128-page freeze/unmap means were 14.11/24.72 us; unchanged map installation was
  46.36 us. These are elapsed observations with host/probe effects, not an isolated
  before/after benchmark or a desktop speedup claim.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
  Interactive F5 media was untouched.
- Default x86 UEFI clippy, x86/ARM64 smoke clippy, layout, formatting and
  whitespace checks pass.
- Logs: `build/region-edit-host.log`, `build/region-edit-host-clippy.log`,
  `build/region-edit-arm64-debug.log`, `build/region-edit-arm64-release.log`,
  `build/region-edit-x86-smoke.log`, `build/region-edit-report.log`,
  `build/region-edit-clippy.log`, `build/region-edit-smoke-clippy.log`,
  `build/region-edit-arm64-clippy.log`, `build/region-edit-desktop.log`.

Next: runtime mapping installation still repeats hierarchy walks per page and
must preserve complete rollback on refusal. Investigate a reserved-table range
transaction before adding suspension state. The authority sweep, deadline
extraction and other audited atomic work remain; guaranteed CPU service is still
separate from ceilings. The intermittent clean-close LOSS timeout remains open.
No owner semantics or application ABI changed.

## Atomic reserved-range installation (2026-10-20)

Baseline: `2771bad8`. Runtime page-object installation now enters `map_pages` in
the [x86](source-rs/kernel/arch/isa/x86_64/user/memory/regions.rs) or
[ARM64](source-rs/kernel/arch/isa/aarch64/user/regions.rs) backend. The first pass
validates all backing addresses, table branches and vacant destination leaves;
the second installs ordinary RW/NX or RO/NX entries. Both passes visit each
leaf table once, giving a maximum-size object two or four hierarchy walks rather
than one per page. No allocation or external work separates the passes.

Refusal changes neither parent nor leaf tables. Core reserves its ledger space
first, then publishes entries only after the complete mapping succeeds. No
partial runtime installation needs rollback. x86 admission now reserves parent
tables with user/write permission using its unpublished scratch alias; runtime
installation refuses restrictive parents instead of widening them. The scratch
leaf is removed before any task runs. Obsolete `NoFrames` and ARM64 single-page
rollback helpers were removed; x86 keeps a private scratch-unmap helper.

This remains an atomic inactive-root operation under the current single-CPU,
IRQs-masked contract. Leaf validation/writes, backing validation and ledger work
still scale with bounded populations. Frame custody, allocation cancellation,
caller charging, final refunds and capability publication retain their existing
ordering. No owner decision or userspace ABI change was needed.

Verification:

- Workspace host tests pass. The expanded x86 tests independently translate RW/NX
  and RO/NX mappings across PT, PD and PDPT boundaries. Complete table snapshots
  remain unchanged for all 128 possible collisions, a missing later table,
  read-only parents, a huge-page conflict and invalid frames. Retrying after
  refusal succeeds; neighbors and table counts remain unchanged.
- x86 release smoke and ARM64 debug/release pass. The shared
  [real-frame proof](source-rs/kernel/core/user_tasks/address_space/regions_test.rs)
  injects all 128 possible conflicts across two tables, checks that core adds no
  ledger entries on refusal, removes only the conflict and successfully retries.
  Invalid final backing and all-slot mapping/sealing/removal also pass, with the
  existing exact frame/funding/heap reclamation checks.
- All five periodic wait/selection population checks pass. This run's 128-page
  mapping mean was 19.56 us (max 152 us). Host scheduling and workload mix prevent
  treating this as an isolated speedup or a real-time bound.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
  Interactive F5 media was untouched.
- Arch host clippy, default x86 UEFI clippy, x86/ARM64 smoke clippy, source layout,
  formatting and whitespace checks pass.
- Logs: `build/region-map-host.log`, `build/region-map-arch.log`,
  `build/region-map-host-clippy.log`, `build/region-map-arm64-debug.log`,
  `build/region-map-arm64-release.log`, `build/region-map-x86-smoke.log`,
  `build/region-map-report.log`, `build/region-map-all-phases.log`,
  `build/region-map-clippy.log`, `build/region-map-smoke-clippy.log`,
  `build/region-map-arm64-clippy.log`, `build/region-map-desktop.log`.

Next: inspect image-loading work behind the remaining larger admission samples
before choosing another split. The all-session report observed 895 `ImageBatch`
visits with a 179.62-us mean and 20 samples at least 1 ms; these include mixed
fixtures and host effects and do not establish a cause. Authority withdrawal,
deadline extraction and the other audited atomic paths also remain. Keep the
timing audit honest about complete trap/boot turns versus individual phases.
The intermittent clean-close LOSS timeout remains open; guaranteed CPU service
is still separate from ceilings.

## Bounded image-copy work and page initialization (2026-10-20)

Baseline: `c4351353`. Investigation of the larger `ImageBatch` samples found
redundant writes: the loader zeroed each entire page, then overwrote its file
prefix. [Page population](source-rs/kernel/core/user_tasks/address_space/image.rs)
now copies that prefix directly and zeroes only the tail. A wholly zero BSS or
stack page never reads source contents. Every byte is initialized before cache
maintenance or mapping; ownership, rollback and publication order are unchanged.

New lab-only per-page observations separate backing allocation, initialization
of file-backed pages, initialization of all-zero pages, and cache/translation
publication. They narrowed the larger x86 samples to file-backed initialization;
allocation, zero-only pages and publication were much smaller in the observed
population. Disassembly of the measured build showed word-sized `rep movsq` and
`rep stosq` in its standard copy/fill routines. That does not diagnose the remaining
emulator/host contribution or justify replacing those routines with custom ISA code.

[Runtime construction](source-rs/kernel/core/user_tasks/admission/construction.rs)
now gives [the loader](source-rs/kernel/core/user_tasks/address_space/creation.rs)
two independent allowances: eight initialized/mapped pages and 8 KiB of file bytes
per turn. A page whose prefix would exceed the remaining byte allowance stays at
the cursor until the next turn. Partial file pages spend their actual prefix size;
BSS/stack pages spend only the page allowance. The allowance must accommodate at
least one full page, so a valid next page cannot strand the cursor. Initial boot
admission remains synchronous through the same engine. More turns permit peer
execution but can lengthen total launch time under contention.

`ImageBacking`, `ImageFile`, `ImageZero` and `ImagePublish` are per-page attempts
inside the per-turn `ImageBatch`. File includes a partial page's zeroed tail;
Publish includes cache maintenance and table allocation/leaf installation, not
making a task runnable. Failed allocation is sampled before returning; failed
mapping is also sampled. Counts can therefore differ. Ordinary builds omit these
hooks and clocks, and synchronous initial construction supplies no observer.
The report tool labels the distinct populations. See the
[timing audit](source-rs/kernel/core/scheduler/timing.md) for scope and units.

Verification:

- Workspace host tests pass. All 4,097 possible prefix lengths are tested against
  poisoned destination pages and adjacent sentinels for byte-backed and frame-backed
  contents. Partial final source pages and empty-source BSS pass too. The cursor
  test crosses partial pages, BSS, gaps and stack with both allowances, preserving
  exact address order and progress on every turn.
- x86 release smoke and ARM64 debug/release pass. Every observed admission turn checks
  its copied-byte allowance. The complete real-frame synthetic construction checks
  exactly 72 backing/publication attempts, one file page and 71 zero pages; initial
  synchronous roots are excluded. Existing physical refusal, withdrawal, retained
  source, fault-containment and exact reclamation sweeps pass.
- All five periodic wait/selection checks pass. The instrumented run before the
  copy allowance recorded 903 image turns, mean 165.96 us and 24 samples >=1 ms;
  afterward it recorded 2,443 turns, mean 67.36 us and one sample >=1 ms. These are
  different subdivisions and mixed session populations, not a total-launch speedup
  or a time bound. A single file-page sample still exceeded 1 ms afterward.
- Core host clippy, default x86 UEFI and x86/ARM64 smoke clippy, Python report tests,
  layout, formatting and whitespace checks pass.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media after
  the copy allowance change. Interactive F5 media was untouched.
- Logs: `build/image-load-host.log`, `build/image-load-host-clippy.log`, `build/image-load-clippy.log`,
  `build/image-load-smoke-clippy.log`, `build/image-load-arm64-clippy.log`,
  `build/image-load-x86-smoke.log`, `build/image-load-arm64-release.log`,
  `build/image-load-arm64-debug.log`, `build/image-load-desktop.log`,
  `build/image-load-report.log`. The earlier instrumented run is preserved as
  `build/image-load-before-budget-report.log` and
  `build/image-load-before-budget-serial.log`; the initial copy-routine inspection
  is in `build/image-load-symbols.log` and `build/image-load-disassembly.log`.

Next: the largest copy unit is now one file page. Do not shrink it or introduce
SIMD merely from QEMU maxima; use a controlled source/copy probe if investigating
that remaining cost. Next prioritize capacity-focused authority-withdrawal
evidence: its wider grant scans still run atomically, while the existing subtree
fixture covers nine descendants. Deadline work also remains in the timing audit.
Guaranteed CPU service is still a separate design from ceilings. The intermittent clean-close LOSS
timeout remains open; no owner semantics or userspace ABI changed here.

## Scoped grant withdrawal at task capacity (2026-10-20)

Baseline: `691bc714`. IPC task closure previously scanned the entire grant arena
even though dynamic records already occupy recipient-local reserved ranges.
[Grants](source-rs/kernel/core/ipc/grants.rs) now provides the common range used
by lookup, vacancy search, handle-budget accounting and
[withdrawal](source-rs/kernel/core/ipc/lifecycle.rs). Installation asserts that
dynamic grants use their owner's vacant range and refer to dynamic endpoints.
This includes invisible incoming moves: transfer vacates the source and installs
a new recipient-local record. No ownership index or additional state was added.

At the current 31-task capacity, close scans 192 packed boot records plus the
task's 48 dynamic slots (240 candidates), instead of all 1,680 records. Escrow
discard still walks its actual affected grant chain. Sponsorship and channel
accounting remain independent of custody, with charges held until final closure.
Boot endpoint replacement clears its three fixed records, including an old
revoker. Control-pair acceptance touches six records and finds its peer-local
ordinal in the boot prefix. Local handle-budget queries scan only 48 dynamic
records. Names, visibility, permissions and the application ABI are unchanged.

[Capacity policy tests](source-rs/kernel/core/ipc/tests/withdrawal.rs) fill all
64 boot endpoints and 192 dynamic channels. One case closes the last task slot
with all 48 visible dynamic grants, checks exact discarded-memory notifications,
preserves every unrelated dynamic grant/queue, then restarts with fresh names.
Another fills all 48 recipient slots with a chain of hidden receive rights, with
memory escrow at its tail. Closing the recipient drains the chain, preserves
the surviving sender's sponsorship charge until final close and leaves peers'
queued messages intact. Both cases finish with no dynamic channels left.

The [real-frame subtree fixture](source-rs/kernel/core/user_tasks/lifecycle/withdrawal_test.rs)
now fills all 31 task slots: initial owner, unrelated survivor and 29 descendants
in two delegated branches. Each spawn stays within the existing 14-loan wire
capacity. It queues grandchild data after owner death, then proves every descendant
is stopped and stripped of authority before one survivor completion batch runs.
No canceled child consumes its data; physical cleanup retains slots/outcomes until
collection and returns every frame/funding unit. The cleanup stall guard scales
with task capacity; the full fixture needed 2,130 retirement visits in this run.

Verification:

- Workspace host tests and core host clippy pass, including both full-arena
  scenarios and existing transfer, initial-bundle, restart and sponsorship tests.
- x86 release smoke and ARM64 debug/release pass, including all 31 real roots,
  both delegated branches, survivor delivery and exact reclamation. The x86
  fixture observed its one authority sweep at 125 us; this is a QEMU observation,
  not a bound or an elapsed comparison against the previous nine-descendant case.
- All five periodic wait/selection checks pass. Default x86 UEFI and x86/ARM64
  smoke clippy, layout, formatting and whitespace checks pass.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
  Interactive F5 media was untouched.
- Logs: `build/authority-host.log`, `build/authority-core.log`,
  `build/authority-capacity.log`, `build/authority-host-clippy.log`,
  `build/authority-arm64-release.log`, `build/authority-arm64-debug.log`,
  `build/authority-x86-smoke.log`, `build/authority-report.log`,
  `build/authority-clippy.log`, `build/authority-smoke-clippy.log`,
  `build/authority-arm64-clippy.log`, `build/authority-desktop.log`.

Next: full-capacity cleanup exposes repeated conservation-accounting work:
2,131 observations totaled 19,912 us in this fixture, versus 2,678 us in retirement
bodies. Inspect repeated ledger scans before adding more scheduling machinery,
retaining both physical and funding conservation checks. Authority's remaining
link/launch scans, channel sponsorship scans, escrow chains and simultaneous
deadline extraction still run atomically. No real-time entitlement or owner
semantic was added. The intermittent clean-close LOSS timeout remains open.

## Reuse the cleanup backing count (2026-10-20)

Baseline: `2a1f4b89`. Each cleanup step's
[conservation check](source-rs/kernel/core/user_tasks/lifecycle/reclamation.rs)
counted page-object backing twice against an immutable session. It now counts
once and reuses the result in the physical-allocation and funding equations.
Both assertions still run after every step, with the same task/draft counts,
allocator baseline and unfinished allocation reservations. No maintained counter,
sampling interval or allocation/retirement behavior was introduced.

Verification: workspace host tests, x86 release smoke, ARM64 release portability,
x86 smoke clippy, formatting and whitespace checks pass. The real-frame suites
exercise partial allocation/rollback, retained backing and escrow, and the full
31-task subtree with exact refunds. All five periodic wait/selection checks pass.
Logs: `build/accounting-host.log`, `build/accounting-x86-smoke.log`,
`build/accounting-arm64-release.log`, `build/accounting-clippy.log`, and
`build/accounting-report.log`. No desktop rerun was needed for this local count
reuse; the prior milestone's desktop evidence is unchanged. F5 media was untouched.

The full-capacity x86 fixture still performs 2,131 conservation checks. This run
recorded 13,768 us total versus the previous 19,912 us; retirement itself recorded
4,861 us versus 2,678 us. Host/QEMU timing variation prevents attributing a precise
elapsed speedup. The structural result is one backing scan instead of two.

Next: conservation remains an atomic full recount. Avoid adding cached totals
without an independent audit that catches drift across allocation refusal,
partial cancellation and retained escrow. The timing inventory still identifies
simultaneous deadline extraction and aggregate authority work as remaining
boundaries. No owner choice or ABI changed; the intermittent clean-close LOSS
timeout remains open.

## Four-level desktops and retained-store heap pressure (2026-10-20)

Baseline: `29d5f02e`. The owner reported lag around four nested matrices in
normal F5 with existing users. The desktop and latency runners now accept
`--nested-depth 4`, exercising four nested desktops **beneath** the outer one.
The deepest Notes focus coordinate scales with depth; the first exploratory
four-level failure was a harness click outside that app, corrected before the
passing runs. No kernel scheduling, authority, ABI or page budget changed.

The saved F5 disk was copied, never installed into, repaired or reset in place.
On that copy, automatic collection before a nested subtree copy panicked with
`memory allocation of 8208 bytes failed`. Heap tracing placed the failure before
the copy writer: engine state, its 64 KiB cache and the transient live-record
census exhausted the provider's 56-page heap. This was a real provider crash,
not just slow framebuffer writes. The collector now stores fixed-size identities
inside separate event/manifest/chunk sets rather than separately allocated full
keys. The cache is 32 KiB, leaving more room in the same heap. Pins, authentication,
deletion order, exact quota and immediate durability are unchanged.

Compact identities alone still exhausted the heap. With both changes, the
initial saved-disk replay created and entered the third nested user without a
provider restart; observed requested-allocation peak fell from 215,004 to 172,158
bytes. These allocator samples exclude allocator overhead, stack and reader
pages. They do not establish a universal bound. The full live census still
scales with root metadata; fallible/bounded collection workspace remains open.

**Retained-program caveat:** the same disk contains older nested program
manifests than its current outer template. Two extended replays exited a matrix
with `TIMED_OUT` while a copy was underway, after getting past the original OOM.
One ran alongside the retention test; the second ran alone and also failed.
Do not call the owner's existing tree fixed. Older interaction timeout behavior
is a plausible cause, not yet attributed to an exact executable/call site.
No recursive legacy upgrade was added; the owner does not require one. Fresh
Lab recreates users with current executables, while retaining a backup of old data.

[nested_existing_test.py](tools/boot-harness-rs/nested_existing_test.py) preserves
this diagnostic path: source availability check, byte copy, current private boot
image, no object-store installer, nested login/create, deepest edit/save,
dirty-close refusal and clean teardown across cold boot. It expects one outer
User 1 and compact stock chrome; it deliberately fails on old-program timeouts.
The host hierarchy inspector now prints names beside IDs/parents/manifests.
Use a copy because engine open/close can recover or maintain records.

`trace-storage` adds recursive LOSS/CLOSE send, answer, deferred and stale-reply
events. Incarnations and slots are matrix-local, not globally unique task IDs.
The intermittent clean-close LOSS timeout was not reproduced or declared fixed;
its existing refusal policy and deadlines remain unchanged.

Evidence before the storage fix:

- Four-level current-program pointer stress and cold reboot passed, including
  dirty-close protection, deepest persistence and repeated whole-subtree teardown:
  `build/nested-four-focused.log`.
- Four-level latency/cold reopen passed: `build/nested-four-latency.log`. Fresh
  deepest typing observations were 120–133 ms and save 412 ms; user entry was
  5.2 s. Host injection/capture and tracing perturb these upper bounds. Earlier
  isolated key tracing saw 1–4 guest ticks from source to Notes. This does not
  isolate scheduler delay or claim a host-independent performance threshold.
- Five-second delayed interaction fixtures at four levels passed with pointer
  stress and cold reopen: `build/nested-four-delayed.log`.
- Saved-disk OOM, compact-only OOM and initial successful third-user replay:
  `build/nested-existing-inspect/replay-metrics.log`, `replay-fixed.log`,
  `replay-cache.log` in that directory. Extended retained-program failures:
  `build/nested-existing-final.log`, `build/nested-existing-solo.log`.

Final storage verification: workspace host tests, all 38 storage-lab host tests,
default and trace/delay x86 storage/matrix clippy, 27 Python harness tests,
layout, formatting and whitespace checks pass. The three-boot objects retention
suite passes exact host record comparisons, pins, collection and cold recovery:
`build/nested-retention.log` (artifact `objects/20261021T041418057534Z`). Host logs:
`build/nested-lag-host.log`, `build/nested-storage-host.log`,
`build/nested-default-clippy.log`, `build/nested-final-clippy.log`,
`build/nested-final-python.log`. No ARM64 or kernel smoke rerun: this changes
userspace collection/cache and diagnostics, not kernel or architecture code.

The final storage build also passes four nested desktops under pointer stress
on both fresh and cold boots, including exact deepest saved text, dirty-close
protection, isolation and two complete close/reopen cycles per boot:
`build/nested-four-final.log`, artifact `desktop/20261021T041745289469Z`.
The storage-lab host clippy check passes after the inspector change.
The saved-disk replay runner also passes both boots at depth four on a copy of
that current-program fixture (`build/nested-replay-current.log`, artifact
`nested-existing/20261021T042304676407Z`), including edits, save confirmation,
protected close and clean collection. Its source bytes remain identical. This
validates the replay path without reclassifying the older F5 disk's failure.

Next: bound collection workspace independently of record count and measure
launch/seed phases under retained-store pressure. Preserve the failing saved
copy for exact timeout attribution if needed. Current-program desktop evidence
must stay separate from legacy snapshots and from the still-open LOSS timeout.

## Fresh lab as the F5 default (2026-10-20)

The owner prefers a fresh OS on every F5 launch over retaining old nested
program snapshots. **Cathedral: Build & Run in QEMU** now passes `--fresh`.
The existing reset backs up the disk and receipt under `before-fresh-*`, then
seeds current programs; users must be created again. **Cathedral: Resume Saved
Lab in QEMU** explicitly reuses the disk for persistence work. The bare CLI
without `--fresh` still preserves state. No reset mechanism or migration changed.

Verification: launch JSON and both command arguments checked; all ten existing
database-image tests pass, including reset backup/rollback and busy-disk refusal.
Whitespace checks pass. No guest rerun was needed for selecting the already
tested fresh launch path. The current disk was not reset during this change.

## Bounded collection, one refresh per nested binding and a soak (2026-10-21)

Baseline: `7697b70e`. The owner asked for the previous handoff's three targets in
order: bound collection's memory, stress one long session, and reduce nested
entry delays. The measurements below come from the four-level latency workload
with storage traces on disposable media; F5 media was untouched.

**Collection workspace.** [collection.rs](source-rs/platform/libraries/database-objects/collection.rs)
no longer gathers every head into a vector or every live identity into one set.
A pass marks one record kind at a time inside one window of identities (events
by the low byte of their object number, manifests and chunks by their first
digest byte), reading heads in batches of 32. A window whose marks would exceed
`WORKSPACE_BYTES` (24 KiB at a conservative 64 bytes per mark) is split in two
and marked again, down to 256 windows per kind; a window that still overflows
returns the new `Error::Workspace` with nothing removed. The provider skips that
maintenance turn, records `collect-workspace-exhausted` in every build and lets
exact quota admit or refuse the write; explicit COLLECT answers NO_MEMORY. Chunk
indexes are still validated before any removal, the event pass authenticates
every head's event first, pins are validated for their one revision, and
retirement order is unchanged. `Collection` now reports the windows a pass
computed and its largest marked set, sampled as `collect-pass` under tracing.
Two host tests collect a crowded root with a workspace too small for one window
per kind and reach exactly the unbounded result, and refuse whole with a
workspace too small for one mark.

**Provider heap and cache.** [main.rs](source-rs/platform/services/storage/main.rs)
sizes the heap from the funded private pages: 64 pages stay outside for one
maximal reply plus eight of headroom, and the rest is heap, clamped between the
56 pages the engine has always run in and the 128 pages one private object may
hold. [database.rs](source-rs/platform/services/storage/database.rs) reserves
256 KiB of that heap for engine state, the writer and the collection workspace
and gives the remainder to the engine cache, never below the previous 32 KiB.
The desktop profile now funds the provider with 200 private pages, so its heap
is 512 KiB and its cache 256 KiB (64 pages) instead of 8 pages; every lab
profile keeps its previous 56-page heap and 32 KiB cache. The first attempt
funded 384 pages and asked for a 312-page heap, which the kernel refused as one
object: the provider panicked at startup and the desktop showed its sign-in
recovery screen as designed. Provider heap peaks in the traced four-level runs
were 373 KB (fresh) and 376 KB (cold) of 524,288, with recovery itself peaking
near 370 KB; the heap report line now prints `capacity=`.

**One refresh per nested binding.** The trace showed why entry cost grew faster
than depth: before forwarding any child request, a nested matrix sent
STORAGE_INFO upstream, and the parent answered that only after refreshing in
turn, so one request from the deepest of the nine matrix hops at depth four
cost about eighty transits before reaching the provider. The depth-four save
spent nine ticks on the request path and none on the reply path.
[pending.rs](source-rs/distribution/matrix/service/parent/storage/pending.rs)
now refreshes before the first exchange only, while `storage_epoch` is zero,
and once more when the parent refuses a stamped revision: a child's refusal is
answered after that refresh (its attachment already travelled), while the
matrix's own program read repeats its step. Grants, revocations and provider
replacement still clear bindings exactly as before; a stale revision is still
refused by the parent rather than accepted. In the cold-boot trace the
nested-receive count fell from 3,672 to 683 and root-receive from 1,360 to 170
for the same 293 provider requests; the depth-four app round trip fell from
25 ticks to 8 to 13.

**Control-stream failures wait briefly for the outcome.** The headless
`--matrix-test` recovery scenario had depended on that refresh chatter: with
the matrix always mid-exchange, a leaf exiting during a forwarded control RPC
was deferred until the kernel reported its outcome, and the asker saw
PEER_CLOSED. Without the chatter the matrix answered BUSY before reclamation
finished. [control.rs](source-rs/distribution/matrix/service/children/control.rs)
now keeps such a request pending, polling each tick, until the child's outcome
arrives or `OUTCOME_GRACE_TICKS` (50) pass inside the asker's deadline, then
answers BUSY as before. A live child that merely dropped its control link still
gets BUSY, half a second later; the `--matrix-storage-test` probes wait 100.

**Soak.** `desktop_test.py --soak CYCLES --nested-depth D` runs
[desktop_soak.py](tools/boot-harness-rs/desktop_soak.py): each cycle enters the
nested desktops, checks the text the previous cycle saved, replaces it with a
cycle token, saves, closes the app and tears the subtree down from the outer
taskbar; a cold boot checks the last token and runs two more cycles. It records
per-phase seconds, provider heap live/peak/capacity, request and collection
counts, and fails on replacements, offline storage, matrix exits, workspace
exhaustion, allocation failure or less than 32 KiB of heap headroom. The
nested-desktop helpers moved into a `Nested` class shared with the existing
workload. The harness's first cold boot failed on its own numbering bug (it
counted the fresh boot's cycles while they were still being written and
expected `c12` where `c23` had been saved); the OS was fine. Sixteen cycles at
depth two then passed on a fresh lab and two more after a cold boot:

| Cycle | Seconds | enter-1 | enter-2 | edit | save | close | teardown | Heap live | Heap peak |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 (creates users) | 9.67 | 3.64 | 4.23 | 1.03 | 0.20 | 0.28 | 0.28 | 326,664 | 369,932 |
| 2 to 16 | 7.12 to 8.08 | 2.11 to 2.61 | 2.67 to 2.89 | 1.38 to 1.84 | 0.19 to 0.23 | 0.28 to 0.31 | 0.28 to 0.31 | 322,536 to 339,032 | up to 373,116 |
| cold 1, 2 | 7.64, 9.61 | 2.16, 2.95 | 2.86, 3.78 | 1.81, 1.98 | 0.22, 0.30 | 0.30 | 0.30 | 326,872, 326,920 | up to 376,876 |

Every cycle issued 105 provider requests (165 with user creation), automatic
collection never triggered (the root stayed below its watermark) and no
replacement, exit or exhaustion marker appeared. Edit time rises with the token
length. An earlier twelve-cycle run drifted from 5.5 to 8.7 seconds per cycle
with the same flat heap and request counts; the sixteen-cycle run did not, so
treat that as host variation until a kernel-side per-cycle sample exists.

**Nested entry, before and after**, host-observed milliseconds on the cold
boot of `desktop_test.py --latency --nested-depth 4 --trace-storage`:

| Depth | Launch picker | Enter user | Launch Notes | Save |
| --- | --- | --- | --- | --- |
| 1 | 1,257 to 969 | 1,579 to 1,287 | 490 to 483 | 193 to 225 |
| 2 | 1,757 to 1,100 | 2,480 to 1,359 | 878 to 450 | 211 to 210 |
| 3 | 3,324 to 1,447 | 3,528 to 1,856 | 1,184 to 645 | 395 to 240 |
| 4 | 5,037 to 1,995 | 5,196 to 2,173 | 1,737 to 746 | 333 to 226 |

At depth four the picker launch's provider CPU fell from 1,681 ms to 765 ms and
its sectors read from 3,336 to 1,576; user entry from 1,130 ms to 785 ms. On the
fresh boot, which also installs each level's template closure, depth-four picker
launch fell from 8,306 ms to 3,337 ms, user entry from 5,205 ms to 1,996 ms and
user creation from 1,792 ms to 486 ms (provider CPU 727 ms to 98 ms). Typing
stayed at about 125 ms per key at every depth; it is bounded by injection and
capture, not the guest. One depth-zero Notes close on the fresh boot took
1,341 ms against 250 to 300 ms everywhere else, with nothing in the guest trace
between the save's completion and the next launch; it is recorded, not attributed.
Before: `build/boot-harness-rs/desktop/20261021T040219186638Z`; after:
`build/boot-harness-rs/desktop/20261022T022034263570Z`.

**Fixture repair.** `provider_live_test.py` still clicked the pre-compact
taskbar's left-hand RETRY SAVES position; its exhausted and readiness-stall
cases now click the right-hand column's row.

**Documentation.** The distribution map now says four nested desktops and names
the current launch entries; this handoff's deadline figures match the contract's
1200-tick base; the provider, store and harness guides describe the funded heap,
windowed collection and soak.

Verification:

- Host: workspace `cargo test --locked` (261 tests), `cathedral-database-objects
  --features host` (20, including the two new window tests), storage-lab engines
  (38), harness Python (12 including the soak bookkeeping), clippy for the store,
  provider and matrix in default and trace builds, `check-userspace`, layout,
  formatting and whitespace checks pass.
- Guest, on disposable media: four-level latency on fresh and cold boots; four
  nested desktops under pointer stress with protected close, confined
  persistence, repeated subtree teardown and cold reopen (three runs);
  `objects_test.py --retention`; `matrix_database_test.py --recovery` (twice);
  the plain default desktop with host verification of 553 records; the full
  provider-replacement suite on that fixture (precommit, postcommit, exhausted,
  readiness-stall, nested and deep); `--matrix-storage-test`, `--matrix-test`
  and `--matrix-desktop-test` with the final grace code; the sixteen-cycle soak.
- Logs: `build/soak-latency-four.log`, `build/suite-*.log`, `build/suite2-*.log`,
  `build/suite3-*.log`; soak artifact
  `build/boot-harness-rs/desktop/20261022T024618213478Z`.

Next, in order:

1. Fresh-boot entry is now dominated by each level's first launch installing
   its template closure (nine subtree copies, two copies and two named
   creations, 1.3 to 1.5 s of provider CPU per level) and by the engine extent
   growing from 1.5 MiB to 3.1 MiB during the first user creation, which zeroes
   3,300 sectors (0.8 s). Seed the extent at its working size and measure
   whether the closure can be shared or installed lazily.
2. Bulk program reads still cost 1.2k to 1.6k sectors per level: each 4 KiB
   chunk pulls two to three tree pages through the cache. A larger cache needs
   either a multi-object heap or a page cache outside the allocator; measure the
   hit rate before choosing.
3. Each Nested level is two matrix hops (the user matrix and the nested root).
   The refresh fix made a hop cost one transit each way; the remaining
   per-hop cost is event-loop turns, visible in the cold trace as one to two
   ticks per request at depth four.
4. The intermittent clean-close LOSS timeout did not reproduce in these runs.
   Less storage traffic means fewer turns in which a matrix is `storing()` and
   answers LOSS with BUSY; keep the trace on and attribute the next occurrence.
5. The desktop never reached the collection watermark here, so windowed
   collection ran only under host tests and the guest retention proof. A soak
   that fills a root (larger notes, more users) would exercise it live.
6. The soak samples the provider's heap only. Kernel task-slot, frame and grant
   accounting per cycle would turn the host-side cycle time into evidence.

## The engine owns its region from format (2026-10-21)

Baseline: `6c27d97d`. The owner asked why the storage engine's extent grew at
runtime at all, since the provider is handed one fixed region and Cathedral's
attenuation pattern is exactly that: you get this much, manage inside it. It
grew because redb thinks it writes an ordinary file, and the adapter turned
each file extension into a zeroing pass over the newly claimed range, 3,300
sectors and 0.8 s on the first user creation.

The region is now the engine's from format and nothing is zeroed at runtime.
redb still keeps a layout length and, at open, requires the backend to report
exactly the length it stored, so the envelope keeps mirroring that length: a
change flushes earlier engine writes, then publishes and flushes both envelope
copies, and writes nothing else. Zeroing was never a redb requirement. The
engine never reads space it has not written, and after an unclean stop it
rebuilds its allocators from its trees rather than trusting region headers, so
the bytes a regrown range hands back may be whatever the engine last wrote
there. Pinning the reported length to the capacity was considered and rejected:
redb shrinks its layout at commit whenever the last region's trailing free
space exceeds half, with a crate-private policy, and a stored layout that
differs from the reported length sends every open through its recovery path.
Mirroring the layout keeps every open clean.

Changed: [the guest adapter](source-rs/platform/libraries/storage-volume/engine/backend.rs)
and [the host adapter](tools/storage-lab/engines/volume/backend.rs) publish the
envelope only; the extent observer, `open_observed`, the provider's extent
trace and the timing tools' extent phase are gone. The guest adapter's growth
test now counts the exact device operations of a length change (one flush, two
envelope writes, two flushes) and proves engine bytes untouched across shrink
and regrowth; the host proof checks the same and its crash-cut census fell from
939 to 501 cuts because a length change has fewer operations to cut.

Measurements, host-observed milliseconds on the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`, before and after:

| Step | Observed | Provider CPU |
| --- | --- | --- |
| create-user (first, grows the layout) | 1,143 to 350 | 1,004 to 90 |
| enter-user | 1,096 to 1,014 | 580 to 514 |
| depth-1 launch picker | 2,021 to 1,652 | 1,414 to 1,104 |
| depth-4 launch picker | 3,337 to 3,249 | 1,486 to 1,493 |

No `extent` trace appears in the new run. Every other step moved within host
variation in both directions; typing stayed at about 125 ms per key. An
eight-cycle soak at depth two ran 5.1 to 5.5 s per cycle after its first, with
the provider heap peaking at 374,908 of 524,288 bytes and no failure marker.
Artifacts: `build/boot-harness-rs/desktop/20261022T033219328260Z` (latency),
`20261022T033652312438Z` (soak), `20261022T033332334361Z` (four levels under
pointer stress); logs `build/suite5-*.log`.

Verification: guest adapter tests (4), storage-lab host tests (38) and the
volume crash-cut proof; workspace `cargo test --locked` (261); harness Python
tests; clippy for the provider and adapter; `check-userspace`; formatting and
whitespace checks. Guest, on disposable media: the volume transport proof, the
funded-heap database proof, `objects_test.py --retention`,
`matrix_database_test.py`, the plain default desktop with host verification,
four-level latency on fresh and cold boots, four nested desktops under pointer
stress with cold reopen, and the eight-cycle soak.

Next: the fresh-boot entry cost is now the nested template closure installed
at each level's first launch (nine subtree copies, two copies and two named
creations per level, 1.1 to 1.5 s of provider CPU). Whether that closure
should be seeded or shared is an owner-level question about what a Nested
instance owns; measure the copy work and ask before changing it. Bulk program
reads and the per-hop event-loop cost remain as recorded in the previous entry.

## A matrix configured by its realm (2026-10-22)

Baseline: `241412ad`. The owner's framing, in order: a matrix is a program with
files beneath it; its configuration is that program with nested copies of the
programs it expects, so swapping a nested desktop's chrome is swapping a
program; and the presence or absence of those programs decides what the matrix
does, with no special path for matrices. This entry builds that shape, then
takes the live-collection and nested-fixture items left open by the previous
two entries.

**Realm-decided startup.** [startup.rs](source-rs/distribution/matrix/service/children/startup/startup.rs)
replaces the launch argument, the `WORKSPACE_ARGUMENT` contract and the runtime
template preparation (`realm.rs`, deleted). A hosted matrix reads its realm once
its grant has a width: a `Matrix` configuration makes it a workspace that runs
the saved CONFIGURE path itself; a `Login` picker beside a `Users` folder makes
it a desktop that runs that picker over those users; a realm with neither, or
no binding at all (DENIED or PEER_CLOSED from its parent), leaves it to its
owner's commands. [service.rs](source-rs/distribution/matrix/service/service.rs)
starts in `Startup::Realm` when an upstream parent exists and `Startup::Hosted`
otherwise; the `desktop-host` feature of
[main.rs](source-rs/distribution/matrix/main.rs) only encodes the funding a
desktop needs when it is launched into an app slot. An owner's explicit
CONFIGURE is satisfied by a matrix that already configured itself
([configuration.rs](source-rs/distribution/matrix/service/children/configuration.rs)),
so the boot owner's order no longer races the user matrix's own startup. The
realm selects policy, never authority: every stamp still arrives through the
storage forwarding chain.

**Root login ordering.** The root adapter admitted the chosen user before the
picker retired, so the owner's SURFACE_LIMIT for the retiring picker replaced
the user's grant epoch and its background saw NO_MEMORY on its first placement.
[children.rs](source-rs/distribution/matrix/service/children/children.rs) now
recognizes an owner RUN_OBJECT of the realm's `Login` over its `Users` in slot
zero as the picker, and [launch.rs](source-rs/distribution/matrix/service/children/launch.rs)
retires it before admitting the user it chooses, so the user's first grant is
final; the owner keeps that picker's recovery
([lifetime.rs](source-rs/distribution/matrix/service/children/startup/lifetime.rs)).
The desktop shell's [login state](source-rs/distribution/desktop/session/shell/login.rs)
treats a picker PEER_CLOSED with a live user binding as entry instead of placing
a picker that no longer exists. Surface-create refusals are sampled as
`surface-create-refused` under tracing.

**Prebuilt closure.** A realm entry in the profile is a program or one of three
directives: `@folder`, `@matrix-config` and `@copy=PATH`, a breadth-first copy
of an earlier entry's subtree at revision 1 that numbers siblings before
descendants. [realm_programs.rs](tools/storage-lab/engines/volume/realm_programs/realm_programs.rs)
applies them at install, [desktop.rs](tools/storage-lab/engines/volume/desktop.rs)
models the result independently, `run.py` validates them and
`database_image.py` fingerprints a directive by its text in the installation
receipt. [profile.json](source-rs/distribution/profile.json) installs Nested as
the stock matrix, `Nested/Login`, the `Nested/Users` folder, a `Users/Template`
holding its chrome, Notes, Colors and `Matrix` configuration, and copies of
Nested into each template five levels deep; the fifth template holds no Nested,
so the fourth nested desktop the lab exercises still offers a NESTED tile. A
first launch copies nothing. Creating a user is one subtree copy: 52 objects
beneath the new user for the outer template, 42 for the first nested one. The
plain desktop verifies against the oracle: six base objects, 15 realm entries,
a 53-object user copy and 919 records.

**Collection under pressure.** The store's lab `pressure` feature
([Cargo.toml](source-rs/platform/libraries/database-objects/Cargo.toml)),
reached as `collection-pressure` on the provider and `--collection-pressure` on
`desktop_test.py --soak`, shrinks the workspace to 2 KiB and collects before
every mutation. The six-cycle soak under it ran the windowed collector live in
every cycle: one to three passes, up to 45 windows, no `Workspace` refusal, no
failure marker, the same saved text and a heap peak of 372,996 of 524,288
bytes. Saves took 4 to 10 s per cycle there because every write collects; that
is the feature, not a regression, and the host tests run without it.

**Whole diagnostic lines.** [diagnostics.rs](source-rs/platform/libraries/user-runtime/diagnostics.rs)
sends a message that fits one write as one call even when it straddles a page
in its process, copying through a page-aligned buffer. The kernel prefixes each
call, so the halves of `picker retry exhausted` read as two lines and the
nested-failures fixture failed its count twice before this.

Measurements, host-observed milliseconds on the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`, the previous
entry's figures first:

| Step | Observed | Provider CPU | Sectors read/written |
| --- | --- | --- | --- |
| create-user (outer, 53 objects) | 350 to 332 | 90 to 124 | 33/105 to 57/201 |
| enter-user | 1,014 to 1,003 | 514 to 581 | 1,280/16 |
| depth-1 launch picker | 1,652 to 957 | 1,104 to 560 | 1,192/16 |
| depth-4 launch picker | 3,249 to 1,785 | 1,493 to 812 | 1,560/16 |
| depth-1 create-user (now 42 objects) | 377 to 1,079 | 182 to 856 | 129/209 to 977/737 |
| depth-4 create-user (now 15 objects) | 494 to 840 | 121 to 425 | 73/153 to 449/321 |

Nested picker launches no longer install a template; what remains is reading
the picker's program. Nested user creation got slower because the template it copies is now the whole
closure beneath it, and that copy costs far more per object than the outer
one: the depth-1 copy of 42 objects reads 977 sectors and writes 737 with 856 ms
of provider CPU, where the outer copy of 53 reads 57, writes 201 and costs
124 ms. The cold boot reaches the fourth nested desktop with picker launches of
988 to 1,149 ms and user entries of 1,256 to 2,237 ms; typing stayed at 115 to
145 ms per key at every depth. Four nested desktops under pointer stress reach
each level in 5.4 to 8.9 s. The eight-cycle plain soak at depth two ran 4.9 to
5.9 s per cycle with the heap live between 330 and 343 KB, a peak of 375,180
bytes, no collection pass and no failure marker.

Verification:

- Host: workspace `cargo test --locked` (261); storage-lab with
  `--features engines` (39, including the directive install test); harness Python
  (29); clippy for the stock matrix in default, `trace-storage` and
  `desktop-host` builds, the provider with `collection-pressure`, the desktop
  shell and the user runtime; `python tools/source-layout/check.py`;
  formatting and whitespace checks.
- Guest, on disposable media: the plain default desktop with oracle
  verification; four nested desktops under pointer stress with cold reopen
  (twice); four-level latency on fresh and cold boots; the eight-cycle soak at
  depth two and the six-cycle soak under collection pressure; the
  provider-replacement suite (precommit, postcommit, exhausted, readiness-stall,
  nested, deep) on the plain fixture; nested startup failures (picker, user,
  background) on the stress fixture; `--matrix-storage-test`, `--matrix-test`
  and `--matrix-desktop-test`.
- Logs `build/suite6-*.log`, `build/suite7-*.log`, `build/suite8-*.log`.
  Artifacts: latency `build/boot-harness-rs/desktop/20261022T052816562717Z`,
  soak `20261022T051017179655Z`, pressure soak `20261022T051837283716Z`,
  nested failures `build/boot-harness-rs/nested-failures/20261023T004212925180Z`.

Next, in order:

1. Attribute the nested user copy. Forty-two objects at depth one cost seven
   times the IO of fifty-three at the root. Candidates: the cache is cold after
   the picker's 1.2k-sector program read, or each copied object re-proves its
   scope through the nested stamps. The `provider-begin`, `provider-end` and
   `ata-io` samples in the latency serial logs hold the answer; add a copy
   label if they do not.
2. Bulk program reads: 1.2k to 1.6k sectors per picker launch, unchanged, as
   recorded two entries ago.
3. Each Nested level is still two matrix hops; the per-hop event-loop cost
   stands as recorded.
4. The clean-close LOSS timeout did not reproduce in these runs either.
5. The soak samples the provider heap only; kernel task-slot, frame and grant
   accounting per cycle remain to be added.
6. Launching one program over one realm twice is now an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md). The Nested tile launches the single
   installed matrix, and no lab fixture launches it twice.

## One-page chunks and a two-object provider heap (2026-10-22)

Baseline: `65d7528b`. The previous entry's first item asked why a nested user
copy of 42 objects cost seven times the IO of the outer copy of 53. The latency
run's per-request samples answer it: the whole cost is one COPY_TREE request,
and its reads vary with what the engine cache holds, not with the subtree.
Every picker launch reads a program through the same 256 KiB cache and evicts
the metadata pages the next copy needs. Two things made that worse than it had
to be, and both are fixed here.

**A chunk record fills one engine page.** A 4 KiB chunk, its 34-byte key and
redb's leaf bookkeeping exceeded one 4 KiB page, so redb stored every chunk as
a single-large-value leaf of two pages: a program read moved twice its bytes
through the cache and the stock matrix's 257 KB read moved a megabyte.
[lib.rs](source-rs/platform/libraries/database-objects/lib.rs) sets
`CHUNK_BYTES` to 4,032 so the record fits one page; the manifest bound and the
pin marks use `div_ceil`, the host oracles in the storage lab chunk with the
same constant, and a new store test publishes 32 chunks and checks the table's
fragmented bytes stay under a kilobyte per chunk. The plain desktop still
matches the oracle record for record.

**The provider heap spans two private objects.** [user-heap](source-rs/platform/libraries/user-heap/lib.rs)
now manages up to four extents, allocating first-fit through them in order and
returning each free to the extent that holds it; a third host test covers
overflow into a second extent, frees returning home, overlap and the extent
limit. [heap.rs](source-rs/platform/libraries/user-runtime/heap.rs) installs as
many private objects as the per-object cap requires and releases an object
whose installation fails. The provider's heap is capped at two objects so one
of its four slots stays free for the pages behind a bulk reply, and the desktop
profile funds it with 328 pages: a 1 MiB heap and a 768 KiB cache instead of
512 KiB and 256 KiB.

**A kernel bookkeeping ceiling.** The first run with two heap objects failed its
first bulk reply with NO_MEMORY although the provider had 72 pages and 84 frames
to spare. [address_space.rs](source-rs/kernel/core/user_tasks/address_space/address_space.rs)
bounded a task's mapped backing at its image, stack and two full page objects,
so the heap's two objects left nothing for a reply mapping. The ceiling now
covers every private slot plus two reader mappings; it bounds bookkeeping, not
funding, and every allocation still charges its account. The provider records
`bulk-reply-refused` in every build with the page count, the error and its page
and frame budgets, the soak treats that marker as a failure and the timing
summary accepts it.

**Per-cycle kernel accounting.** After a matrix reclaims a child subtree,
[lifecycle.rs](source-rs/distribution/matrix/service/children/lifecycle.rs)
samples `matrix-reclaimed` under `trace-storage`: the slot, the private and
reader pages the kernel still charges this account, its charged frames and the
launch slots free again. The soak records the cycle's last sample, which is the
root's teardown of the Nested tile, and fails if it differs from the first
cycle's. Across three cycles and a cold boot's two the root settled at 0 private
and 38 reader pages, 104 frames and 22 free slots every time, so a leaked slot,
page or frame now shows as drift rather than as a slower cycle.

Measurements, host-observed milliseconds on the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`: the previous
entry, then one-page chunks alone, then both changes.

| Step | Observed | Provider CPU | Sectors read |
| --- | --- | --- | --- |
| enter-user | 1,003 / 808 / 460 | 581 / 306 / 164 | 1,280 / 592 / 568 |
| depth-1 launch picker | 957 / 662 / 374 | 560 / 267 / 107 | 1,192 / 544 / 360 |
| depth-4 launch picker | 1,785 / 1,476 / 949 | 812 / 496 / 40 | 1,560 / 976 / 64 |
| depth-1 create-user | 1,079 / 873 / 278 | 856 / 600 / 62 | 977 / 585 / 1 |
| depth-4 create-user | 840 / 818 / 377 | 425 / 395 / 56 | 449 / 401 / 25 |
| depth-4 enter-user | 2,195 / 2,313 / 1,178 | 667 / 490 / 69 | 1,296 / 968 / 104 |

The nested copy now reads one sector at depth one and 25 at depth four: the
template's pages stay cached across the picker's program read, which itself
moved 360 sectors instead of 1,192 because the chooser's chunks are one page
each and partly shared with the root picker. Picker launches at depth four cost
40 ms of provider CPU instead of 812. The four-cycle soak at depth two ran 4.0
to 4.2 s per cycle (4.9 to 5.9 before), entering each level in 0.9 to 1.3 s,
with the provider heap live at 843 to 858 KB and peaking at 900,132 of
1,048,576 bytes: the engine cache fills to its budget, and the 32 KiB headroom
assertion holds with 148 KB to spare. In the per-stage spans of the new run the
nested dispatch stage averages 0.04 ticks, nested service 0.80, root service
0.58 and the provider request beneath them 0.51; an app's storage round trip
averages 3.9 ticks against 11.5 before. The cold boot's depth-4 user entry took
2,880 ms with 158 ms of provider CPU and 40 sectors read, so what remains there
is not storage.

Verification:

- Host: workspace `cargo test --locked` (262, with the new heap and chunk
  tests); storage-lab with `--features engines` (39); harness Python (30);
  `cargo check-userspace`, `cargo check-uefi`, clippy for the provider with
  `collection-pressure` and the store with `host`; formatting, layout and
  whitespace checks.
- Guest, on disposable media: the plain default desktop with oracle
  verification (twice, with the chunk change alone and with both); four-level
  latency on fresh and cold boots (twice); four nested desktops under pointer
  stress with cold reopen; nested startup failures (picker, user, background);
  the provider-replacement suite (precommit, postcommit, exhausted,
  readiness-stall, nested, deep); `objects_test.py` plain and `--retention`;
  `matrix_database_test.py --recovery`; `--matrix-storage-test`,
  `--matrix-test` and `--matrix-desktop-test`; the four-cycle soak at depth
  two, plain and under collection pressure; a three-cycle soak with the
  accounting check (`build/soak-accounting.log`).
- Logs `build/suite9-*.log` (chunks alone), `build/suite11-*.log` and
  `build/suite12-*.log` (both changes); the two refused boots are
  `build/probe-plain.log` and `build/probe-plain2.log`.

Next, in order:

1. Attribute the cold boot's depth-4 user entry: 2,880 ms observed, 158 ms of
   provider CPU, 40 sectors. The latency JSON carries input and redraw samples
   beside the storage ones; admission of the user matrix and its first
   placement through four hops are the candidates.
2. Each Nested level is two matrix hops, but the hop itself is cheap now:
   in the one-page-chunk run the nested dispatch stage averaged 0.05 ticks,
   nested service 2.8 ticks against 2.4 for the provider request beneath it.
   The remaining per-level time is the provider's own work on program reads
   and copies, plus admission.
3. The clean-close LOSS timeout did not reproduce in these runs either.
4. The soak now holds the root's kernel accounting steady across cycles; the
   nested matrices' samples are recorded but not yet compared, and nothing
   samples the provider's or the desktop's own accounts.
5. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## Fewer round trips per nested level (2026-10-22)

Baseline: `96d0e23c`. With the engine cache no longer thrashing, the cold boot's
depth-4 user entry still took 2,880 ms with 158 ms of provider CPU and 40
sectors. The tick-stamped trace shows why: a request from depth four crosses
eight matrices each way and costs 6 to 7 ticks, and a user entry issued about
36 of them in sequence, each waiting for the last. Two sources were cheap to
remove.

**One request per program.** A program launch read HEAD for the revision and
length, then BULK for the bytes. [BULK](source-rs/platform/contracts/storage.rs)
now accepts an expected revision of zero, the current one, and a length of
zero, the whole object from the offset; [bulk.rs](source-rs/platform/services/storage/bulk.rs)
takes the length from the head inside the one request and the reply states the
revision and length it read. The matrix's [program read](source-rs/distribution/matrix/service/parent/storage/pending.rs)
and its [configuration and launcher reads](source-rs/distribution/matrix/service/children/configuration.rs)
issue that single request; the HEAD step is gone. Partial reads and BULK_PIN
keep their explicit arguments.

**One request per tile.** The chooser listed executables with FILTER, then read
both metadata fields of each, two round trips per tile, to learn names whose
type the filter had already proved. [storage.rs](source-rs/platform/libraries/user-runtime/storage.rs)
gains `name`, one METADATA round trip, and the [chooser](source-rs/distribution/applications/chooser/session/realm.rs)
uses it; `metadata` still reads both fields and refuses a rename that lands
between them.

Measurements, host-observed milliseconds and provider request spans on
`desktop_test.py --latency --nested-depth 4 --trace-storage`, the previous
entry's figures first:

| Step | Observed | Provider requests |
| --- | --- | --- |
| enter-user (fresh) | 460 to 441 | 27 to 19 |
| depth-1 launch picker (fresh) | 374 to 359 | 11 to 9 |
| depth-4 launch picker (fresh) | 949 to 629 | 18 to 16 |
| depth-4 enter-user (fresh) | 1,178 to 820 | 29 to 22 |
| depth-4 enter-user (cold) | 2,880 to 821 | 29 to 22 |
| create-user, any depth | 278 to 377, now 278 to 363 | 7 to 6 |

Seven requests left each user entry and one left each user creation, and the
cold boot's depth-4 entry now matches the fresh boot's. Typing stayed at 105 to
118 ms per key. One cold-boot cluster at depth two (picker 1,570 ms, entry
2,388 ms, Notes 2,314 ms) ran with 8 to 24 sectors and under 90 ms of provider
CPU per step and no refusal, retry or stale-reply marker in either boot; the
other three depths were ordinary, so it reads as host variance rather than a
guest wait, and the next run should say.

Verification:

- Host: workspace `cargo test --locked` (262); `cargo check-userspace`;
  formatting and whitespace checks. No host test covers the provider's bulk
  path; the guest suites do.
- Guest, on disposable media (`build/suite13-*.log`): four-level latency on
  fresh and cold boots; the plain default desktop with oracle verification;
  four nested desktops under pointer stress with cold reopen; nested startup
  failures (picker, user, background); the provider-replacement suite on the
  plain fixture; `--matrix-storage-test`, `--matrix-test` and
  `--matrix-desktop-test`; `objects_test.py` plain and `--retention`;
  `matrix_database_test.py --recovery`; the three-cycle soak at depth two,
  whose cycles issue 64 provider requests instead of 85 with the root's
  accounting steady at 0 private and 38 reader pages, 104 frames and 22 free
  slots.

Next, in order:

1. A depth-4 user entry still issues 22 requests in sequence, each a 6 to 7
   tick round trip through eight matrices: the realm shape takes three
   LOOKUPs and the configuration a fourth for the same `Matrix` object, the
   chrome lists executables then names each tile. A realm read that answers
   several names at once, or a shape lookup carried into configure, would cut
   the sequence further; the per-hop cost itself is now 0.04 ticks of dispatch
   plus the turn.
2. The clean-close LOSS timeout did not reproduce in these runs either.
3. The soak holds the root's kernel accounting steady across cycles; nested
   matrices' samples are recorded but not yet compared, and nothing samples the
   provider's or the desktop's own accounts.
4. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## A realm named in one listing (2026-10-22)

Baseline: `9b073f4d`. The previous entry left 22 sequential requests in a
depth-4 user entry, most of them the chooser naming its tiles one METADATA at a
time and the matrix looking its realm up one name at a time.

**ENTRIES.** [storage.rs](source-rs/platform/contracts/storage.rs) gains a
nineteenth operation: `parent, 0, after, 0` answers the direct children after
the cursor, in number order, as one sealed page of `Entry` rows (number,
revision, Metadata; 51 per page), with the count and the last listed number in
the packet and no page for an empty listing. It is a read like LIST and FILTER,
so a scope admits it when it admits the parent, and the matrices forward its
reply page as they forward a bulk read's. [objects.rs](source-rs/platform/libraries/database-objects/objects.rs)
gathers the rows in one transaction and a host test checks order, cursors,
bounded output, an empty parent and a missing one; [listing.rs](source-rs/platform/services/storage/listing.rs)
fills the page beside the bulk reply path.

**One listing per reader.** [storage.rs](source-rs/platform/libraries/user-runtime/storage.rs)
reads a page of entries and resolves several names from one listing. The
[chooser](source-rs/distribution/applications/chooser/session/realm.rs) builds
its tiles from the realm's executables, finds the template or the background by
name and the launcher by the name the configuration record carries, all from
one listing plus one READ: two requests where a six-tile launcher spent
fourteen. The matrix's [realm shape](source-rs/distribution/matrix/service/children/startup/startup.rs)
is one listing instead of three LOOKUPs, and it carries the configuration it
found into [configure](source-rs/distribution/matrix/service/children/configuration.rs)
instead of looking it up again.

**A program must fit one object.** The first guest run of the listing panicked
the provider with `Corrupt` on its first ENTRIES. The store had never been
asked for the hidden `Template` entry's head before, and that head's length was
269,856 bytes: the first reader of the listing carried page-sized `Entry`
arrays and a per-row decode loop that grew the stock matrix by 14 KB of code,
past the 256 KiB object limit, and the installer had written the oversized head
without complaint. The reader now decodes rows on demand from the sealed page
(`Listing`), which puts the matrix back at 257,568 bytes, and the installer's
[seed.rs](tools/storage-lab/engines/volume/seed.rs) refuses any program over
`MAX_BYTES` with its size in the message, covered by a lab test, so the next
such growth fails at install rather than as a corrupt head on first read.

**A chooser is lent two reader pages.** The second guest run refused the first
listing's page at the root matrix with DENIED: pickers and launchers had always
been funded with zero reader pages, since nothing they read came back as an
attachment. [matrix.rs](source-rs/distribution/matrix/contracts/matrix.rs) names
`CHOOSER_READERS`, two pages, and the shell's root picker, the nested picker and
the configured launcher are lent that beside their private pages.

**Size headroom.** The stock matrix is now built at `opt-level = "s"`
([Cargo.toml](source-rs/Cargo.toml)): 220,704 bytes trimmed instead of 257,568,
leaving 16 percent of the object limit rather than two. The four-level latency
run measured no change: nested service averaged 0.84 ms of CPU per request
against 0.87 before, root service 0.96 against 0.92, typing 107 to 123 ms per
key either way, and every step moved within host variation. Four nested
desktops under pointer stress, the nested startup failures, the three matrix
suites and a three-cycle soak pass on it (`build/suite17-*.log`), the soak's
root settling at 95 frames after each teardown instead of 105 with the smaller
image. `run.py` prints each trimmed program's size as a share of the limit at
every build and refuses one over it before the installer does.

Measurements, host-observed milliseconds and provider request spans on
`desktop_test.py --latency --nested-depth 4 --trace-storage`, the previous
entry's figures first:

| Step | Observed | Provider requests |
| --- | --- | --- |
| enter-user (fresh) | 441 to 365 | 19 to 9 |
| depth-1 launch picker (fresh) | 359 to 354 | 9 to 5 |
| depth-4 launch picker (fresh) | 629 to 510 | 16 to 12 |
| depth-4 enter-user (fresh) | 820 to 617 | 22 to 13 |
| depth-4 enter-user (cold) | 821 to 645 | 22 to 13 |
| create-user, any depth | 278 to 363, now 283 to 295 | 6 to 3 |

A user entry at depth four now issues thirteen requests where the entry two
steps back issued twenty-nine, and takes 617 ms where it took 1,178 on a fresh
boot and 2,880 on a cold one. A user creation is three requests: the copy, the
listing and the configuration read.

Verification:

- Host: workspace `cargo test --locked` (262); the store's host tests with
  `--features host` (22, with the entries test); storage-lab with
  `--features engines` (40, with the oversized-program refusal);
  `cargo check-userspace`; formatting, layout and whitespace checks.
  The stock matrix trims to 257,568 bytes and the chooser to 78,008.
- Guest, on disposable media (`build/suite16-*.log`): four-level latency on
  fresh and cold boots; the plain default desktop with oracle verification;
  four nested desktops under pointer stress with cold reopen; nested startup
  failures (picker, user, background); the provider-replacement suite on the
  plain fixture; `--matrix-storage-test`, `--matrix-test` and
  `--matrix-desktop-test`; `objects_test.py` plain and `--retention`;
  `matrix_database_test.py --recovery`; the three-cycle soak at depth two,
  whose cycles issue 36 provider requests instead of 64 and run 3.2 to 3.5 s
  with the root's accounting steady at 0 private and 38 reader pages, 105
  frames and 22 free slots. The two refused runs are `build/suite14-*.log` (the
  oversized matrix) and `build/suite15-*.log` (no reader pages).

Next, in order:

1. A depth-4 user entry still issues 13 sequential requests, each a 6 to 7
   tick round trip through eight matrices: the matrix's listing, two bulk
   reads for the configuration and the launcher, the background's lookup and
   read, the launcher's listing and configuration read, and the picker
   retirement around them. The background's lookup could ride the listing the
   matrix already read, and a launcher could receive its arrangement with its
   launch instead of reading it; beyond that the hop count itself is the cost.
2. The stock matrix is 220,704 bytes against a 262,144-byte object limit and
   every build prints the margin. If it grows past that, the limit would have to
   grow with the provider's reply pages, which is a budget decision across the
   provider, the matrices' reader allowances and the kernel's backing ceiling.
3. The clean-close LOSS timeout did not reproduce in these runs either.
4. The soak holds the root's kernel accounting steady across cycles; nested
   matrices' samples are recorded but not yet compared, and nothing samples the
   provider's or the desktop's own accounts.
5. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## Where a forwarded request spends its time (2026-10-22)

Baseline: `44b11710`. With the request count per nested level cut, the hop
itself is what remains, and ticks were too coarse to see inside one. This entry
adds the clock to see it and reports what it shows.

**A microsecond clock in every stage trace.** [CLOCK_READ](source-rs/kernel/contracts/user.rs)
with a 24-byte record writes the tick, the kernel's microsecond clock behind it
(the TSC-derived clock the scheduler already accounts with) and this task's
measured execution, in one read; [time.rs](source-rs/platform/libraries/user-runtime/time.rs)
uses it for `storage_trace`, so every stage line carries the microsecond clock
as its fifth value and costs two syscalls and a write instead of five and a
write. The timing parsers accept the fifth value. Each matrix writes two
`trace-calibrate` stamps back to back at startup, so the cost of a trace line
itself is in the log beside what it measures.

**hop_latency.py.** [The tool](tools/boot-harness-rs/hop_latency.py) pairs
consecutive stages of a request chain in serial order and reports each
transition's count, total, mean, median, median net of the calibration and p90:
a send to the next receive is the hop (kernel delivery plus the receiving
matrix's wake), receive to admit and admit to send are that matrix's own turn,
and the provider's begin to end is its work. A unit test covers the pairing,
a backwards clock and the calibration.

Measurements, microseconds from the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`
(`build/latency-stamp.log`), medians with the trace line's own 231 µs
subtracted, then raw p90:

| Transition | Count | Net median | Raw p90 |
| --- | --- | --- | --- |
| hop up, matrix to matrix | 301 | 293 | 782 |
| hop down, matrix to matrix | 334 | 406 | 1,351 |
| hop up, root to provider | 133 | 322 | 673 |
| hop down, provider to root | 72 | 267 | 781 |
| matrix dispatch (receive to admit) | 425 | 22 | 378 |
| matrix forward (admit to send) | 437 | 8 | 338 |
| root dispatch and forward | 233 | 10 and 2 | 324 and 280 |
| provider work | 88 | 406 | 27,719 |

The cold boot reads the same within 30 µs on every row. So a hop costs 300 to
400 µs under TCG, where the matrix's own work on a request is about 30 µs: the
time is kernel delivery plus the receiving matrix's wake and switch, not matrix
code. A depth-4 request crosses eight or nine hops each way, sixteen to
eighteen in all, for about 6 ms, and the thirteen requests of a user entry
about 85 ms of its 581; the rest is admission, placement and drawing. The downward hop costs more than the upward
one because a matrix does more after answering than after forwarding before it
reaches its wait.

**Yield after forward and reply.** The scheduler gives a woken task its turn
when the running one blocks, yields or the timer fires, so a matrix's loop tail
sat between every hop. [pending.rs](source-rs/distribution/matrix/service/parent/storage/pending.rs)
now yields right after forwarding a request upstream and right after answering
a child (`build/latency-yield.log`): the hop down from the root to a matrix fell
from 222 to 54 µs net and matrix to matrix from 406 to 368, the depth-4 user
entry from 581 to 519 ms and the depth-1 picker launch from 354 to 280. The hop
up stayed at about 290 µs net, so what remains is the kernel's switch itself
under emulation: two syscall transitions and a page-table switch, with the
yield's return through the boot context when deferred work is pending. That is
a property of the lab's single CPU under TCG far more than of the design, and
the right fix, if one is wanted, is in the kernel's switch path rather than in
any matrix.

**Kernel phases.** The smoke session's report (`run.py --smoke --release`; the
suite takes 42 s here, so the default boot timeout is now 120 s instead of 30)
puts
the kernel's own work far below the hop: a trap averages 5 µs and a syscall
2 µs across 190,000 of each, and a task whose registered wait completes is
selected 8 to 13 µs later with a yielding peer. The IPC lab's echo client now
times its thirty-two round trips with a clock grant (`ipc-roundtrip`): 3,004
and 2,957 µs in the two rounds, so a round trip between two small tasks costs
94 µs and a switch about 45, four syscalls included. The matrix hop's 290 µs is
therefore neither trap handling, scheduler latency nor the switch itself: it
is the woken matrix re-touching its working set, static arenas and mapped
objects alike, through the emulator's translation cache after the page-table
reload, before it reaches its stamp. Real hardware pays a few microseconds
for all of it. A direct switch to the woken task would save the yield's own
trap, about 5 µs; the hop count, not the hop, is the lab's lever, and that
count is now what the realm layout requires.

**Two older items close.** Bulk program reads are no longer amplified: a user
entry reads the 220 KB stock matrix in 496 sectors, 1.15 times its size, where
it read 1,280 for 257 KB before the one-page chunks. The clean-close LOSS
timeout left no trace in this session's 310 desktop runs: no stale control
reply, the three storage retirements are the provider panics of the two
refused listing builds, and the 23 close refusals all report a sibling that
answered with unsaved work, which is the protected-close check the tests
exercise on purpose.

Verification:

- Host: workspace `cargo test --locked` (262); `cargo check-uefi` and the
  kernel's host tests (144) for the clock read; `cargo check-userspace`; harness
  Python (31, with the hop tool's test); formatting, layout and whitespace
  checks.
- Guest, on disposable media: four-level latency on fresh and cold boots three
  times (`build/latency-micros.log`, `build/latency-stamp.log`,
  `build/latency-yield.log`); then with the yield (`build/suite19-*.log`) the
  plain default desktop with oracle verification, four nested desktops under
  pointer stress with cold reopen, nested startup failures, the
  provider-replacement suite, `--matrix-storage-test`, `--matrix-test` and
  `--matrix-desktop-test`, and the three-cycle soak at depth two with the root's
  accounting steady at 0 private and 38 reader pages, 95 frames and 22 free
  slots; `build/suite18-*.log` covers the clock read alone on the plain desktop
  and the two input-traced matrix suites. `build/soak-accounts.log` is the
  three-cycle soak with every reclamation sample compared: each cycle, in both
  boots, reports the two picker retirements, the nested desktop's Notes close
  and the root's teardown with identical pages, frames and free slots. The
  later changes in this section each ran the same suites again:
  `build/suite20-*.log` (background in the configure listing),
  `build/suite21-*.log` (held close deadline), `build/suite22-*.log` (launch
  argument), `build/soak-custody.log` (provider custody floor) and
  `build/soak-shell.log` (shell account floor); `build/smoke-roundtrip.log` is
  the smoke suite with the timed echo exchange. A soak cycle now issues 32
  provider requests, from 85 at the start of the day.

Next, in order:

1. The per-hop cost is about 290 µs under TCG: a bare switch between two
   small tasks measures 45 µs, the kernel's trap work 5 µs and selection
   within 13 µs of completion, so the rest is the woken matrix re-touching its
   working set through the emulator's translation cache. The receive path was
   tried as a lever: a stamp at the moment the wait returned
   (`build/latency-wake.log`, `build/latency-wake2.log`) put the whole up hop
   before the wake and none of it in the matrix's turn, and showed the server
   rescanning every client after a wake; the wait now points the poll at the
   channel the kernel reported ready and a scan reads the clock once
   ([polling.rs](source-rs/platform/libraries/user-runtime/server/polling.rs)),
   which removes a receive per quiet client per wake but leaves the hop at
   294 µs net (`build/latency-poll3.log`) against 287 to 293 before. The
   per-turn stamp itself cost 250 µs a turn and was retired. The refill's
   origin is in [entry.S](source-rs/kernel/arch/isa/x86_64/interrupts/entry.S):
   every trap switches to the kernel's own root on entry and back to the
   task's root on return, and no mapping is global
   ([inherited.rs](source-rs/kernel/arch/isa/x86_64/memory/inherited.rs)
   strips the flag, [creation.rs](source-rs/kernel/arch/isa/x86_64/user/memory/creation.rs)
   masks it), so every syscall drops both the kernel's and the task's
   translations and the emulator re-walks them: 45 µs for a tiny task, far
   more for a matrix. Global supervisor leaves (the seven identical ranges and
   the slot-unique task stacks) would spare the kernel's half; keeping the
   task's half needs PCID or the kernel running in the task's root, which is
   the isolation boundary the owner has left open. Both are recorded under
   that question in OWNER_QUESTIONS.md with this evidence; a direct switch to
   the woken task is the CPU sponsorship question there. Nothing in the
   matrices or the server is left to shave.
2. A depth-4 user entry issues 11 sequential requests, each with its reason
   (`build/latency-poll3.log`): one BULK reading the user matrix's program;
   four HEADs, one per nesting level, each matrix's first-time proof that the
   new binding's scope lies within the one it was granted, which the shared
   scope machinery then retains for every later request; one ENTRIES for the
   realm's shape; one BULK for the configuration; one ENTRIES naming the
   launcher and the background; two BULKs for their programs; and the
   launcher's own ENTRIES for its tiles. The root entry is the same without
   the proofs, seven requests, and a depth-1 entry eight. The background rides
   the configure listing (`build/suite20-latency-four.log`) and a program the
   matrix runs from its realm receives its own object above a role byte in
   its launch argument ([matrix.rs](source-rs/distribution/matrix/contracts/matrix.rs)),
   so the launcher hides itself without a read (`build/suite22-latency-four.log`).
   The proofs are the trust model, one per matrix that forwards; the rest is
   the realm's content. The entry's remaining time is admission, placement and
   drawing.
3. The clean-close LOSS timeout: the mechanism is in
   [control.rs](source-rs/distribution/matrix/service/children/control.rs). A
   close's answer is left queued while this matrix's storage is busy, and the
   deadline used to keep running meanwhile, so the first poll after a long
   busy window could refuse a sibling that had answered cleanly as timed out.
   The pending deadline is now held at least the outcome grace ahead of now
   for as long as the matrix is busy, so only the sibling's own silence can
   time a close out. It left no trace in this session's runs before or after;
   `build/suite21-*.log` covers the fix on the close, failure, provider and
   soak suites.
4. The soak now holds every matrix's reclamation sample steady across cycles,
   in order: the nested desktop's after closing Notes and the root's after the
   teardown. The provider's heap report carries the private pages and frames
   the kernel charges it, which include reply pages their readers still hold,
   and the soak holds each cycle's quietest level: 256 pages and 533 frames,
   the two heap objects alone, in every cycle of both boots
   (`build/soak-custody.log`). The shell samples its own account about once a
   second while it idles under storage tracing (`shell-account`), and the soak
   holds its floor too: no private or reader pages, 46 frames and no free
   launch slots in every cycle of both boots (`build/soak-shell.log`). Every
   account in a session now has a per-cycle check.
5. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## The hop on hardware, and global shared leaves (2026-10-22)

Baseline: `f8bab679`. The section before this one attributed the per-hop cost
under TCG to the two root switches of every trap and named global supervisor
leaves as the one lever that needs no boundary decision. This entry measures
the hop where paging is the host CPU's, takes that lever, and leaves the lab
able to time either way.

**TCG cannot show global pages.** QEMU's `cpu_x86_update_cr3` flushes the whole
software TLB on every CR3 write while paging is on, `cpu_x86_update_cr4`
flushes again only when PGE itself changes, and the x86 page walker never reads
the global bit (v9.2.0 and current master alike). Under TCG a global leaf
therefore costs nothing and saves nothing, and the 45 µs a bare switch costs
there is the emulator rebuilding its own TLB, not the guest's. The lab's QEMU
(11.0.50) also carries WHPX, the Windows hypervisor accelerator, under which
the guest's page tables and TLB run on the host CPU. [run.py](tools/boot-harness-rs/run.py)
and [desktop_test.py](tools/boot-harness-rs/desktop_test.py) take `--accel whpx`.
The guest cannot see QEMU's VM port under WHPX, so the desktop runs there with
`--relative-pointer`, which the latency and soak workloads now accept, and the
latency workload's pointer steps go through the transport-aware `point` helper
instead of sending absolute positions themselves. The smoke suite and the
depth-4 latency workload pass under WHPX
(`build/whpx-smoke.log`, `build/boot-harness-rs/desktop/20261023T060538222615Z`).

**What hardware pays.** Every serial byte is a hypervisor exit under WHPX, so a
trace line costs about 1,150 µs there against 231 under TCG, and the hop figures
net of it sit inside its own noise: 0 to 140 µs for the hop up between matrices
and 39 to 224 down over two boots, against 290 and 370 under TCG. A traced
workload is therefore bound by its own tracing under WHPX, and the untraced
entry is the end-to-end figure. All TCG figures here are today's: by the
afternoon the host ran TCG about 1.6 times slower than in the morning's runs,
the trace line itself at 434 to 649 µs against 231, on the unchanged kernel as
well (820 to 959 ms for the traced depth-4 entry before the global
leaves, `build/boot-harness-rs/desktop/20261023T063432217407Z`), so a figure compares only with one taken
beside it.

| Figure, 32 round trips or one depth-4 step | TCG | WHPX |
| --- | --- | --- |
| bare round trip between two tasks, fastest burst | 3,140 to 4,824 µs | 118 to 144 µs |
| the whole exchange, eight bursts | 36,977 to 44,346 µs | 1,094 to 1,217 µs |
| trace line itself | 434 to 649 µs (231 in the morning) | about 1,150 µs |
| hop up between matrices, net median | 332 to 512 µs (290 in the morning) | within trace noise |
| depth-4 user entry, traced | 828 to 851 ms | 787 to 989 ms |
| depth-4 user entry, untraced | 620 to 629 ms | 329 to 431 ms |
| depth-4 picker launch, untraced | 460 to 533 ms | 777 to 785 ms |

(`build/final*-smoke-*.log`, `build/boot-harness-rs/desktop/20261023T063541759172Z`, `build/boot-harness-rs/desktop/20261023T060759118274Z`, `build/boot-harness-rs/desktop/20261023T063026021739Z`,
`build/boot-harness-rs/desktop/20261023T062823481668Z`.) Two emulation costs shape the WHPX column. Every word
of an ATA PIO transfer is a port-I/O exit, so the steps that read programs from
disk are slower there: the root entry's 496 sectors take 1,655 ms against 553,
the depth-1 picker launch's 296 take 1,362 against 464, while the depth-4 steps
read 16 sectors or none and show the kernel and the matrices instead. And the
desktop runs on the relative pointer there, so a step that clicks carries the
pointer's paced walk, about 40 ms per 70 pixels, which the TCG run on the
absolute pointer does not pay; the depth-4 picker launch is mostly that walk.
What the kernel and the matrices add on hardware is small beside both.

**Global shared leaves.** [shared.rs](source-rs/kernel/arch/isa/x86_64/memory/shared.rs)
names the supervisor ranges every root maps identically, the image, the boot
and emergency stacks and the context heap, and marks the image's identity
leaves global in the kernel root after [inherited.rs](source-rs/kernel/arch/isa/x86_64/memory/inherited.rs)
has copied the firmware's tables; the stack and heap leaves are mapped global,
and [creation.rs](source-rs/kernel/arch/isa/x86_64/user/memory/creation.rs)
copies the flag into every user root with the leaf. Activation retires the
firmware's global translations and sets CR4.PGE; the user session asserts it
instead of clearing it. Nothing else is global: the firmware's aliases of
image frames and the per-slot task stacks stay per root and are retired by
each switch, so a restricted root inherits no translation from the TLB that it
does not map, and no flush site changes, since the shared ranges never change
after boot. A host test walks a copied tree and checks the identity leaves are
global, the alias of the same frames is not and the neighbours are untouched.
Measured as an interleaved A/B, three smoke runs each way under each
accelerator with a 256-round exchange: under WHPX 1,310 to 1,359 µs without
the global leaves and 1,128 to 1,175 with (one run 3,508 with a host
preemption inside), about 15 percent less per round trip, 5.2 to 4.5 µs; under
TCG 38.4 to 38.5 ms without and 36.9 to 42.0 with, unchanged within noise, as
QEMU's source says it must be; on the desktop workload under TCG, run beside
each other, the traced depth-4 entry took 820 to 959 ms without and
828 to 851 with. Kept: correct by construction, no isolation
cost, and the kernel's half of every switch spared on hardware. The task's
half still needs PCID or the kernel executing in the task's root, which is the
isolation boundary the owner has left open; both evidence paragraphs in
OWNER_QUESTIONS.md carry the hardware figures now.

**Where the rest of the entry goes.** [startup_timeline.py](tools/boot-harness-rs/startup_timeline.py)
lists one matrix startup from its calibration stamps: every request of its
chain that reached the provider, in milliseconds after it started, until its
launcher names the tiles. For the depth-4 matrix of today's traced TCG run
(`build/boot-harness-rs/desktop/20261023T063541759172Z`, boot 1) the four
scope proofs are answered by 83 ms, the realm's shape at 86, the configuration
at 117, the launcher's and background's names at 161, their programs at 195 and
252, the background is admitted and the matrix configured by about 280, and the
launcher's tiles are named at 315; the next request, at 701 ms, is the Notes
launch the harness clicks afterwards. So the matrix's startup is about 315 ms
of the 828 ms step and the launcher's admission about 28 of them; the rest is
the launcher drawing, the compositor, and the harness itself, whose every
clicked step carries 210 ms of input spacing and a 50 ms pause between
screendumps, which is why the record calls its figures upper bounds. Under
WHPX the same chain reads 538 ms traced, nearly all of it the chain's own five
hundred trace lines at 1.15 ms each; untraced, the whole step is 329 to 431 ms
with the pointer's walk inside it, so on hardware the entry sits at the
harness's floor.

**The held close deadline, pinned.** The clean-close fix in
[control.rs](source-rs/distribution/matrix/service/children/control.rs), which
keeps a close answer's deadline at least the outcome grace ahead while this
matrix's storage is busy, had only non-reproduction behind it. The test-only
[delay probe](source-rs/distribution/matrix/service/delay_probe.rs) lets F4
travel on from the root, and the matrix on the focused chain that receives it
from its parent arms its next close to count its storage as busy for two
seconds, exchange or none, longer than the answer's 100-tick deadline and
shorter than the chrome's 600-tick wait for the close; that close keeps its
deadline from the moment it was asked but sends its LOSS only when the window
ends ([lifecycle.rs](source-rs/distribution/matrix/service/children/lifecycle.rs)),
so the sibling's clean answer lands after the busy window and inside the
grace, which is the only timing the fix decides: a matrix polls a close's
control channel without blocking, so an answer already queued is read however
late the poll. The interaction-delay suite closes Notes in the outer desktop
from the taskbar under that window (a window's own control is the app leaving
by itself, no close at all; and that desktop is the user matrix under the
storage root, which is why the root cannot arm for it): the answer is accepted
and the close completes
(`build/hold-fixed.log`); with the extension removed the same step fails
(the late answer is refused as timed out, Notes stays, and the desktop never reports closed and collected; `build/hold-unfixed.log`), so the scenario detects the bug it
pins. Two earlier shapes could not: a busy window armed before the close
never sees one, because a matrix defers its children's requests while its
storage is busy, so the chrome's close waited for the window to end; and an
unrelated exchange at the root would have been a launch, which the root
refuses at admission beside a live three-deep subtree. Counting the storage as
busy from the close itself, with nothing in flight, is what makes the timing
reachable.

**The exchange in bursts.** 256 round trips under TCG cost 150 µs each against
93 for 32, because ten-millisecond ticks land inside a 38 ms exchange. The echo
client now runs eight bursts of 32, times each, and reports the fastest burst
beside the whole exchange (`ipc-roundtrip 32 <fastest> <total>`); the fastest
burst is the switch cost with no tick or host preemption inside it.
[lab.rs](source-rs/kernel/contracts/lab.rs) holds the counts for the kernel's
lab and the program alike, so the lab's accounting check and the program agree
by construction.

Verification:

- Host: workspace `cargo test --locked` (263); `cargo check-uefi` and the
  arch crate's host tests (13, with the shared-leaves test); `cargo
  check-userspace`; harness Python (12, with the startup timeline's);
  formatting and layout checks.
- Guest: the interaction-delay suite with the storage stall, passing with the
  held deadline and failing with it removed (`build/hold-fixed.log`,
  `build/hold-unfixed.log`); the smoke suite under TCG and WHPX twice each on
  the final tree
  (`build/final-smoke-*.log`); the interleaved A/B (`build/ab-*.log`); the
  depth-4 latency workload with traces under WHPX before and after the global
  leaves (`build/boot-harness-rs/desktop/20261023T060538222615Z`, `build/boot-harness-rs/desktop/20261023T060759118274Z`)
  and under TCG before and after (`build/boot-harness-rs/desktop/20261023T063432217407Z`, `build/boot-harness-rs/desktop/20261023T063541759172Z`); the
  untraced depth-4 workload under both (`build/boot-harness-rs/desktop/20261023T062823481668Z`,
  `build/boot-harness-rs/desktop/20261023T063026021739Z`).

Next, in order:

1. The switch path has nothing left to take without a boundary decision: the
   kernel's translations survive every trap, the task's would need PCID or the
   kernel in the task's root. PCID is possible on this host under WHPX
   (`-cpu qemu64,+pcid,+invpcid` is accepted) and impossible under TCG, whose
   CR3 write rejects the no-flush bit and whose TLB flushes whole regardless
   (QEMU source), so it would be a hypervisor-only kernel path with
   hypervisor-only validation. Both live under the isolation-boundary question
   in OWNER_QUESTIONS.md with the TCG and WHPX figures, as the owner's call.
2. Under TCG the startup chain is hop-bound, and the one lever left inside it
   was the four scope proofs: each the first request's proof, at the matrix
   that cannot add a third stamp, that the innermost scope lies within the
   other two, answered one after another in the first 83 ms of a depth-4
   startup. Warming them was tried (2026-10-23): the granting matrix sent a
   HEAD on the new scope through the child's binding right after the grant,
   before the child asked anything. On the lab's one CPU that chain runs
   ahead of the child's deferred first request instead of beside it, and adds
   a request of its own: the depth-4 chain measured 357 to 385 ms with
   eleven provider requests against 312 to 365 with ten in alternating runs
   (`build/warm-*.log`), and no entry step moved, so the code is gone again.
   OWNER_QUESTIONS.md keeps the trust question (a parent exercising a
   child's binding) with these figures; only a host where the chain could
   overlap the child's start would change them, and there a hop costs
   microseconds.
3. Timing taken under one accelerator compares only with timing under the
   same one. The TCG figures in the sections before this one stand; a figure
   for hardware is a WHPX run of the same command.
4. The owner questions this work raised remain open: launching one program
   over one realm twice, CPU sponsorship across service calls, the isolation
   boundary, and a parent warming its child's proofs.


## CPU budget decision: archived lab evidence (2026-10-23)

The owner has accepted [executing-program CPU charging](wiki/spec/resources/execution_budgets.md). The following evidence was moved from the closed owner question to retain its measurements and implementation links. Its historical references to unresolved CPU sponsorship are superseded by that contract; caller-funded execution is deferred. Other memory, capability and accounting limitations remain limitations. No tests were rerun for this documentation change.

**Rust experiment (not resolution).** [Hierarchical turns](source-rs/kernel/core/scheduler/README.md)
now follow current launch ownership. Descendants divide their owner's outer turns;
loans reattach dormant scheduling accounts, and reclamation returns them without
resetting rotation. A provider still runs within its own launch tree even when
serving another tree. The QEMU proof compares one versus three spinning descendants
against an outside account. The measured-budget extension now charges elapsed
execution, spawn attempts, page operations and reclamation through the ancestor
chain. Boot-selected burst/refill ceilings retain overrun debt through restart
and loan transfer; QEMU checks spinners, yields, repeated replacement and idle
refill. IRQ handling while a process is selected is charged to that process;
general boot/idle housekeeping has no application sponsor. These are provisional
attribution choices and ceilings, not guaranteed CPU-time reservations. The
periodic timer, unpreemptible kernel work and lab clock do not establish hard
response bounds. Shared-provider request funding remains the unresolved choice
above; merely registering a provider confers no extra scheduling authority.


**Forwarding-hop lab evidence (2026-10-22, not a scheduling decision).** With
a microsecond clock in the storage traces, one hop of a nested storage request
from a matrix to its parent costs about 290 µs under TCG net of tracing, while
the matrix's own work on the request is about 30 µs; a depth-4 request crosses
sixteen to eighteen hops. The scheduler gives a woken receiver its turn only
when the running task blocks, yields or the timer fires, so matrices now yield
right after forwarding or answering, which shortens the downward hops but
leaves the upward hop near 290 µs. The kernel's trap work is 5 µs, a woken task
is selected within 13 µs of its completion, and a bare round trip between two
small tasks measures 94 µs in the IPC lab, so the rest is the woken matrix
re-touching its working set through the emulator's translation cache after the
page-table reload; on hardware (the Windows hypervisor on the lab host) the
same 32 round trips take about 118 to 144 µs and the hop sits inside the
trace line's own noise. A direct switch to the task an IPC wakes would therefore
save only the yield's trap, and what it decides is whose turn funds the
receiver's work, which is this question; the lab keeps the yield and leaves the
switch policy open.

**Resource-subdivision lab evidence (not a production sponsorship decision).**
[The Rust resource coordinator](source-rs/kernel/core/user_tasks/resources/README.md) now
conserves runtime page/reader/channel allowances across owner-to-child assignment.
Only owned idle launch accounts can be resized, and every family validates before
commit. Accepted pages now retain the original backing sponsor after producer
death; restart cannot buy replacement capacity while a reader pins it. Explicit
withdrawal fails rather than revoking the reader. The same applies to channels.
This extends the persistent-sponsor experiment above. It does not settle funding
transfer or active-child rebudgeting. The bounded physical-memory experiment
below now includes ELF, page-table and stack admission costs.


**Executable-admission lab evidence (not a stable delegation contract).**
[Dynamic admission](source-rs/kernel/core/user_tasks/admission/pool/README.md) now separates
code authority from a bounded process pool. Exporting an owned plain launch
creates a distinct code grant; dynamic admission explicitly supplies that code
grant to its child for funded recursive launch. Device-granted launches cannot
be exported. Creator death uses the existing nearest-surviving-lender policy for
new accounts too, while retained backing can pin returned reservations. Production
semantics still need to decide general executable-capability transfer/revocation
and whether code grants should be retained independently of launch ownership.
The current lab grants permission to execute again; it does not infer that right
from arbitrary readable bytes or service registration.


**Matrix integration experiment.** Code-only boot catalogue grants no longer
consume dormant process accounts. An owner may explicitly stage copied executable
authority into an idle child; admission publishes fresh local names after load.
The matrix desktop uses this path recursively. This adopts copying for the lab,
not a production revocation contract: changing the launch's staging does not revoke
an independently installed grant, and selected dynamic code remains executable
by descendants within supplied process/resource budgets. General IPC transfer,
revocation and production sponsorship remain the choices above. Idle-child
CPU ceiling assignment preserves ancestor enforcement and debt; it confers no
priority or permission to charge an unrelated service's work to a caller.

**Physical-memory funding experiment (not a production sponsorship decision).**
The [Rust frame bank](source-rs/kernel/core/user_tasks/resources/physical/README.md) now backs
conserved allowances with real reserved RAM. Executable pages, stacks, private
page tables and runtime buffers charge the same account. Retained readers keep
the original producer's physical charge after process death; final release refunds
it. Kernel heap/IPC arenas and boot/device reservations remain separate bounded
costs. Nested exhaustion cannot spend a sibling's reservation.

Bootstrap sizing is provisionally 256 frames per bound account plus private-page
quota, and another 256 per unbound process slot initially held by the root. This
eager, fixed session bank is lab policy, not the eventual manifest contract.
Production still needs the already-open choice between persistent sponsors,
explicit funded adoption, and revocation, plus a policy for growing/reclaiming
active memory envelopes. No new answer is needed to continue this bounded lab.


## Isolation decision: archived trap-path evidence (2026-10-23)

The isolation decision is settled by the contract linked above. The following
historical measurements are retained from the closed question; its discussion
of the isolation boundary as undecided is superseded. PCID validation coverage
remains a separate deferred choice, and these results were not rerun here.

**Trap-path lab evidence (2026-10-22, not a boundary decision).** The Rust lab
keeps the kernel in its own page-table root: every trap switches roots on entry
and back on return. The seven supervisor ranges every root shares (image,
stacks, context heap) are global, so the kernel's translations survive the
switches; the task's are dropped by each one. 32 bare round trips between two
tiny tasks cost about 3,140 to 4,824 µs under TCG, where the emulator rebuilds its
whole TLB on every root switch whatever the guest marks global, and about
118 to 144 µs under the Windows hypervisor on the lab host, 15 percent less
than before the global leaves. A storage hop between matrices
is about 290 µs net of tracing under TCG and within the trace line's own noise
on hardware; a depth-4 user entry spends about 85 ms of its 520 in hops under
TCG and almost nothing on hardware. Sparing the task's half needs PCID, which
the lab refuses, or the kernel executing in the task's root, which is this
boundary. PCID itself lies within the boundary, and the lab host can run it:
WHPX exposes `pcid` and `invpcid` on the i9-12900K without complaint. It
cannot run under TCG at all: QEMU's `helper_write_crN` sends a long-mode CR3
write with the no-flush bit set down its reserved-bits path, and its TLB
flushes whole on every root switch regardless (`target/i386/tcg/system/misc_helper.c`,
`target/i386/helper.c`, current master). A PCID kernel path would therefore
run, and be validated, only under the hypervisor, never in the emulator the
suites and the F5 flow use, and its TLB coherence across user roots, which
TCG would hide, is the kind of error only hardware shows. Whether a kernel
path validated only under WHPX is acceptable is the owner's call beside the
boundary. The lab leaves the roots as they are until that is chosen.
